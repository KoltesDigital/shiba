mod settings;

pub use self::settings::MsvcSettings;
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
	settings: &'a MsvcSettings,
	shader_declarations: &'a String,
	shader_loading: &'a String,
	uniform_arrays: &'a [UniformArray],
}

pub struct MsvcCompiler<'a> {
	project_descriptor: &'a ProjectDescriptor<'a>,
	settings: &'a MsvcSettings,

	cpp_template_renderer: cpp::TemplateRenderer,
	glew_path: PathBuf,
	msvc_command_generator: cpp::msvc::CommandGenerator,
	tera: Tera,
}

impl<'a> MsvcCompiler<'a> {
	pub fn new(
		project_descriptor: &'a ProjectDescriptor,
		settings: &'a MsvcSettings,
	) -> Result<Self, String> {
		let cpp_template_renderer = cpp::TemplateRenderer::new(&project_descriptor.configuration)?;
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

impl Compiler for MsvcCompiler<'_> {
	fn compile(&self, options: &CompileOptions) -> Result<PathBuf, String> {
		let contents = self.cpp_template_renderer.render(
			options.project_codes,
			options.shader_descriptor,
			self.project_descriptor.development,
			"executable",
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

		fs::write(TEMP_DIRECTORY.join("executable.cpp"), contents.as_bytes())
			.map_err(|_| "Failed to write to file.")?;

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
			.current_dir(&*TEMP_DIRECTORY)
			.spawn()
			.map_err(|err| err.to_string())?;

		let status = compilation.wait().map_err(|err| err.to_string())?;
		if !status.success() {
			return Err("Failed to compile.".to_string());
		}

		Ok(TEMP_DIRECTORY.join("executable.exe"))
	}
}

impl ExecutableCompiler for MsvcCompiler<'_> {}
