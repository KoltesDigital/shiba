mod settings;

pub use self::settings::Settings;
use crate::code_map::CodeMap;
use crate::configuration::Configuration;
use crate::generator_utils::cpp;
use crate::paths::TEMP_DIRECTORY;
use crate::traits;
use crate::types::{CompilationDescriptor, Pass, ShaderDescriptor, UniformArray};
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
	settings: &'a Settings,
	shader_declarations: &'a String,
	shader_loading: &'a String,
	uniform_arrays: &'a [UniformArray],
}

pub struct Generator<'a> {
	cpp_template_renderer: cpp::TemplateRenderer,
	crinkler_path: PathBuf,
	msvc_command_generator: cpp::msvc::CommandGenerator,
	settings: &'a Settings,
	tera: Tera,
}

impl<'a> Generator<'a> {
	pub fn new(settings: &'a Settings, configuration: &'a Configuration) -> Result<Self, String> {
		let cpp_template_renderer = cpp::TemplateRenderer::new(configuration)?;
		let crinkler_path = configuration
			.paths
			.get("crinkler")
			.ok_or("Please set configuration key paths.crinkler.")?
			.clone();
		let msvc_command_generator = cpp::msvc::CommandGenerator::new()?;

		let mut tera = Tera::default();

		tera.add_raw_template("template", include_str!("../executable/template.tera"))
			.map_err(|err| err.to_string())?;

		Ok(Generator {
			cpp_template_renderer,
			crinkler_path,
			msvc_command_generator,
			settings,
			tera,
		})
	}
}

impl<'a> traits::Generator for Generator<'a> {
	fn generate(
		&self,
		audio_codes: &CodeMap,
		compilation_descriptor: &CompilationDescriptor,
		project_codes: &CodeMap,
		shader_descriptor: &ShaderDescriptor,
	) -> Result<(), String> {
		let contents = self.cpp_template_renderer.render(
			project_codes,
			shader_descriptor,
			self.get_development(),
			"crinkler",
		)?;

		let context = Context {
			api: &contents.api,
			audio_codes: &audio_codes,
			development: self.get_development(),
			opengl_declarations: &contents.opengl_declarations,
			opengl_loading: &contents.opengl_loading,
			passes: &shader_descriptor.passes,
			project_codes: &project_codes,
			render: &contents.render,
			settings: self.settings,
			shader_declarations: &contents.shader_declarations,
			shader_loading: &contents.shader_loading,
			uniform_arrays: &shader_descriptor.uniform_arrays,
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
			.args(&compilation_descriptor.cl.args)
			.arg("&&")
			.arg(&self.crinkler_path)
			.arg("/OUT:crinkler.exe")
			.args(&self.settings.crinkler.args)
			.args(&compilation_descriptor.crinkler.args)
			.arg("crinkler.obj")
			.current_dir(&*TEMP_DIRECTORY)
			.spawn()
			.map_err(|err| err.to_string())?;

		let status = compilation.wait().map_err(|err| err.to_string())?;
		if !status.success() {
			return Err("Failed to compile.".to_string());
		}

		Ok(())
	}

	fn get_development(&self) -> bool {
		false
	}

	fn get_path(&self) -> PathBuf {
		TEMP_DIRECTORY.join("crinkler.exe")
	}
}
