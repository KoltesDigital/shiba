mod settings;

pub use self::settings::CrinklerSettings;
use super::ExecutableCompiler;
use crate::code_map::CodeMap;
use crate::compiler::{CompileOptions, Compiler};
use crate::generator_utils::cpp;
use crate::paths::TEMP_DIRECTORY;
use crate::types::{Pass, ProjectDescriptor, UniformArray};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use tera::Tera;

#[derive(Serialize)]
struct Context<'a> {
	api: &'a String,
	audio_codes: &'a CodeMap,
	development: bool,
	opengl_declarations: &'a String,
	opengl_loading: &'a String,
	passes: &'a [Pass],
	project_codes: &'a CodeMap,
	render: &'a String,
	settings: &'a CrinklerSettings,
	shader_declarations: &'a String,
	shader_loading: &'a String,
	uniform_arrays: &'a [UniformArray],
}

pub struct CrinklerCompiler<'a> {
	project_descriptor: &'a ProjectDescriptor<'a>,
	settings: &'a CrinklerSettings,

	cpp_template_renderer: cpp::TemplateRenderer,
	crinkler_path: PathBuf,
	msvc_command_generator: cpp::msvc::CommandGenerator,
	tera: Tera,
}

impl<'a> CrinklerCompiler<'a> {
	pub fn new(
		project_descriptor: &'a ProjectDescriptor,
		settings: &'a CrinklerSettings,
	) -> Result<Self, String> {
		let cpp_template_renderer = cpp::TemplateRenderer::new(&project_descriptor.configuration)?;
		let crinkler_path = project_descriptor
			.configuration
			.paths
			.get("crinkler")
			.ok_or("Please set configuration key paths.crinkler.")?
			.clone();
		let msvc_command_generator = cpp::msvc::CommandGenerator::new()?;

		let mut tera = Tera::default();

		tera.add_raw_template("template", include_str!("../msvc/template.tera"))
			.map_err(|err| err.to_string())?;

		Ok(CrinklerCompiler {
			project_descriptor,
			settings,

			cpp_template_renderer,
			crinkler_path,
			msvc_command_generator,
			tera,
		})
	}
}

impl<'a> Compiler for CrinklerCompiler<'a> {
	fn compile(&self, options: &CompileOptions) -> Result<PathBuf, String> {
		let contents = self.cpp_template_renderer.render(
			options.project_codes,
			options.shader_descriptor,
			self.project_descriptor.development,
			"crinkler",
		)?;

		let context = Context {
			api: &contents.api,
			audio_codes: &options.audio_codes,
			development: self.project_descriptor.development,
			opengl_declarations: &contents.opengl_declarations,
			opengl_loading: &contents.opengl_loading,
			passes: &options.shader_descriptor.passes,
			project_codes: &options.project_codes,
			render: &contents.render,
			settings: self.settings,
			shader_declarations: &contents.shader_declarations,
			shader_loading: &contents.shader_loading,
			uniform_arrays: &options.shader_descriptor.uniform_arrays,
		};
		let contents = self
			.tera
			.render("template", &context)
			.map_err(|_| "Failed to render template.")?;

		fs::write(TEMP_DIRECTORY.join("crinkler.cpp"), contents.as_bytes())
			.map_err(|_| "Failed to write to file.")?;

		let mut compilation = self
			.msvc_command_generator
			.command(cpp::msvc::Platform::X86)
			.arg("cl")
			.arg("/c")
			.args(&self.settings.cl.args)
			.arg("/FA")
			.arg("/Facrinkler.asm")
			.arg("/Focrinkler.obj")
			.arg("crinkler.cpp")
			.args(&options.compilation_descriptor.cl.args)
			.arg("&&")
			.arg(&self.crinkler_path)
			.args(vec![
				"/ENTRY:main",
				"/OUT:crinkler.exe",
				"/REPORT:crinkler.html",
				"gdi32.lib",
				"kernel32.lib",
				"opengl32.lib",
				"user32.lib",
			])
			.args(&self.settings.crinkler.args)
			.args(&options.compilation_descriptor.crinkler.args)
			.arg("crinkler.obj")
			.current_dir(&*TEMP_DIRECTORY)
			.spawn()
			.map_err(|err| err.to_string())?;

		let status = compilation.wait().map_err(|err| err.to_string())?;
		if !status.success() {
			return Err("Failed to compile.".to_string());
		}

		Ok(TEMP_DIRECTORY.join("crinkler.exe"))
	}
}

impl ExecutableCompiler for CrinklerCompiler<'_> {}
