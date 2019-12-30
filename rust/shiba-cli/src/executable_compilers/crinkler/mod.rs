mod settings;

pub use self::settings::CrinklerSettings;
use super::ExecutableCompiler;
use crate::build::BuildTarget;
use crate::code_map::CodeMap;
use crate::compiler::{CompileOptions, Compiler};
use crate::generator_utils::cpp;
use crate::hash_extra;
use crate::paths::BUILD_ROOT_DIRECTORY;
use crate::types::{Pass, ProjectDescriptor, UniformArray};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use tera::Tera;

pub struct CrinklerCompiler<'a> {
	project_descriptor: &'a ProjectDescriptor<'a>,
	settings: &'a CrinklerSettings,

	cpp_template_renderer: cpp::template::Renderer,
	crinkler_path: PathBuf,
	msvc_command_generator: cpp::msvc::CommandGenerator,
	tera: Tera,
}

impl<'a> CrinklerCompiler<'a> {
	pub fn new(
		project_descriptor: &'a ProjectDescriptor,
		settings: &'a CrinklerSettings,
	) -> Result<Self, String> {
		let cpp_template_renderer =
			cpp::template::Renderer::new(&project_descriptor.configuration)?;
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

const OUTPUT_FILENAME: &str = "crinkler.exe";

#[derive(Hash)]
struct Inputs<'a> {
	cpp_template_renderer: cpp::template::RendererInputs<'a>,
	crinkler_path: &'a Path,
	development: bool,
	msvc_command_generator: cpp::msvc::CommandGeneratorInputs<'a>,
	options: &'a CompileOptions<'a>,
	settings: &'a CrinklerSettings,
}

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

impl<'a> Compiler for CrinklerCompiler<'a> {
	fn compile(&self, options: &CompileOptions) -> Result<PathBuf, String> {
		let inputs = Inputs {
			cpp_template_renderer: self.cpp_template_renderer.get_inputs(),
			crinkler_path: &self.crinkler_path,
			development: self.project_descriptor.development,
			msvc_command_generator: self.msvc_command_generator.get_inputs(),
			options,
			settings: self.settings,
		};
		let build_cache_directory = hash_extra::get_build_cache_directory(&inputs)?;
		let build_cache_path = build_cache_directory.join(OUTPUT_FILENAME);

		if build_cache_path.exists() {
			return Ok(build_cache_path);
		}

		let contents = self.cpp_template_renderer.render(
			options.project_codes,
			options.shader_descriptor,
			self.project_descriptor.development,
			BuildTarget::Executable,
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

		let build_directory = BUILD_ROOT_DIRECTORY
			.join("executable-compilers")
			.join("crinkler");
		fs::create_dir_all(&build_directory).map_err(|err| err.to_string())?;

		let source_path = build_directory.join("executable.cpp");
		fs::write(&source_path, contents.as_bytes()).map_err(|_| "Failed to write to file.")?;

		let mut compilation = self
			.msvc_command_generator
			.command(cpp::msvc::Platform::X86)
			.arg("cl")
			.arg("/c")
			.args(&self.settings.cl.args)
			.arg("/FA")
			.arg("/Faexecutable.asm")
			.arg("/Foexecutable.obj")
			.arg("executable.cpp")
			.args(&options.compilation_descriptor.cl.args)
			.arg("&&")
			.arg(&self.crinkler_path)
			.args(vec![
				"/ENTRY:main",
				"/OUT:executable.exe",
				"/REPORT:report.html",
				"gdi32.lib",
				"kernel32.lib",
				"opengl32.lib",
				"user32.lib",
			])
			.args(&self.settings.crinkler.args)
			.args(&options.compilation_descriptor.crinkler.args)
			.arg("executable.obj")
			.current_dir(&*build_directory)
			.spawn()
			.map_err(|err| err.to_string())?;

		let status = compilation.wait().map_err(|err| err.to_string())?;
		if !status.success() {
			return Err("Failed to compile.".to_string());
		}

		fs::copy(build_directory.join("executable.exe"), &build_cache_path)
			.map_err(|err| err.to_string())?;

		Ok(build_cache_path)
	}
}

impl ExecutableCompiler for CrinklerCompiler<'_> {}
