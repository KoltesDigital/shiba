mod settings;

pub use self::settings::MsvcSettings;
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

pub struct MsvcCompiler<'a> {
	project_descriptor: &'a ProjectDescriptor<'a>,
	settings: &'a MsvcSettings,

	cpp_template_renderer: cpp::template::Renderer,
	glew_path: PathBuf,
	msvc_command_generator: cpp::msvc::CommandGenerator,
	tera: Tera,
}

impl<'a> MsvcCompiler<'a> {
	pub fn new(
		project_descriptor: &'a ProjectDescriptor,
		settings: &'a MsvcSettings,
	) -> Result<Self, String> {
		let cpp_template_renderer =
			cpp::template::Renderer::new(&project_descriptor.configuration)?;
		let glew_path = project_descriptor
			.configuration
			.paths
			.get("glew")
			.ok_or("Please set configuration key paths.glew.")?
			.clone();
		let msvc_command_generator = cpp::msvc::CommandGenerator::new()?;

		let mut tera = Tera::default();

		tera.add_raw_template("template", include_str!("./template.tera"))
			.map_err(|err| err.to_string())?;

		Ok(MsvcCompiler {
			project_descriptor,
			settings,

			cpp_template_renderer,
			glew_path,
			msvc_command_generator,
			tera,
		})
	}
}

const OUTPUT_FILENAME: &str = "msvc.exe";

#[derive(Hash)]
struct Inputs<'a> {
	cpp_template_renderer: cpp::template::RendererInputs<'a>,
	development: bool,
	glew_path: &'a Path,
	msvc_command_generator: cpp::msvc::CommandGeneratorInputs<'a>,
	options: &'a CompileOptions<'a>,
	settings: &'a MsvcSettings,
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
	settings: &'a MsvcSettings,
	shader_declarations: &'a String,
	shader_loading: &'a String,
	uniform_arrays: &'a [UniformArray],
}

impl Compiler for MsvcCompiler<'_> {
	fn compile(&self, options: &CompileOptions) -> Result<PathBuf, String> {
		let inputs = Inputs {
			cpp_template_renderer: self.cpp_template_renderer.get_inputs(),
			development: self.project_descriptor.development,
			glew_path: &self.glew_path,
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
			.join("msvc");
		fs::create_dir_all(&build_directory).map_err(|err| err.to_string())?;

		let source_path = build_directory.join("executable.cpp");
		fs::write(&source_path, contents.as_bytes()).map_err(|_| "Failed to write to file.")?;

		let mut compilation = self
			.msvc_command_generator
			.command(cpp::msvc::Platform::X64)
			.arg("cl")
			.arg("/c")
			.arg("/EHsc")
			.arg("/FA")
			.arg("/Foexecutable.obj")
			.arg(format!(
				"/I{}",
				self.glew_path.join("include").to_string_lossy()
			))
			.args(&options.compilation_descriptor.cl.args)
			.arg("executable.cpp")
			.arg("&&")
			.arg("link")
			.arg("/OUT:executable.exe")
			.args(&self.settings.link.args)
			.arg(
				self.glew_path
					.join("lib")
					.join("Release")
					.join("x64")
					.join("glew32s.lib")
					.to_string_lossy()
					.as_ref(),
			)
			.args(&options.compilation_descriptor.link.args)
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

impl ExecutableCompiler for MsvcCompiler<'_> {}
