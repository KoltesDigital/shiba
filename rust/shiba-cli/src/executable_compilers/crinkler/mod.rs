mod settings;

pub use self::settings::CrinklerSettings;
use super::ExecutableCompiler;
use crate::build::{BuildOptions, BuildTarget};
use crate::compiler::{CompileOptions, Compiler};
use crate::cpp_utils;
use crate::hash_extra;
use crate::paths::BUILD_ROOT_DIRECTORY;
use crate::project_data::Project;
use crate::project_files::{CodeMap, FileConsumer, IsPathHandled};
use crate::shader_data::{ShaderSourceMap, ShaderUniformArray};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};

pub struct CrinklerCompiler<'a> {
	project: &'a Project,
	settings: &'a CrinklerSettings,

	api_generator: cpp_utils::api::Generator,
	crinkler_path: PathBuf,
	msvc_command_generator: cpp_utils::msvc::CommandGenerator,
	tera: Tera,
}

impl<'a> CrinklerCompiler<'a> {
	pub fn new(project: &'a Project, settings: &'a CrinklerSettings) -> Result<Self, String> {
		let api_generator = cpp_utils::api::Generator::new(&project.configuration)?;
		let crinkler_path = project
			.configuration
			.paths
			.get("crinkler")
			.ok_or("Please set configuration key paths.crinkler.")?
			.clone();
		let msvc_command_generator = cpp_utils::msvc::CommandGenerator::new()?;

		let mut tera = Tera::default();

		tera.add_raw_template("template", include_str!("../msvc/template.tera"))
			.map_err(|err| err.to_string())?;

		Ok(CrinklerCompiler {
			project,
			settings,

			api_generator,
			crinkler_path,
			msvc_command_generator,
			tera,
		})
	}
}

impl<'a> Compiler for CrinklerCompiler<'a> {
	fn compile(
		&self,
		build_options: &BuildOptions,
		options: &CompileOptions,
	) -> Result<PathBuf, String> {
		const OUTPUT_FILENAME: &str = "crinkler.exe";

		#[derive(Hash)]
		struct Inputs<'a> {
			api_generator: cpp_utils::api::GeneratorInputs<'a>,
			crinkler_path: &'a Path,
			development: bool,
			msvc_command_generator: cpp_utils::msvc::CommandGeneratorInputs<'a>,
			options: &'a CompileOptions<'a>,
			settings: &'a CrinklerSettings,
			target: BuildTarget,
		}

		let inputs = Inputs {
			api_generator: self.api_generator.get_inputs(),
			crinkler_path: &self.crinkler_path,
			development: self.project.development,
			msvc_command_generator: self.msvc_command_generator.get_inputs(),
			options,
			settings: self.settings,
			target: build_options.target,
		};
		let build_cache_directory = hash_extra::get_build_cache_directory(&inputs)?;
		let build_cache_path = build_cache_directory.join(OUTPUT_FILENAME);

		if !build_options.force && build_cache_path.exists() {
			return Ok(build_cache_path);
		}

		let contents = self.api_generator.generate(
			options.project_codes,
			options.shader_set,
			self.project.development,
			BuildTarget::Executable,
		)?;

		#[derive(Serialize)]
		struct OwnContext<'a> {
			api: &'a String,
			audio_codes: &'a CodeMap,
			development: bool,
			opengl_declarations: &'a String,
			opengl_loading: &'a String,
			project_codes: &'a CodeMap,
			render: &'a String,
			settings: &'a CrinklerSettings,
			shader_declarations: &'a String,
			shader_loading: &'a String,
			shader_specific_sources: &'a ShaderSourceMap,
			shader_uniform_arrays: &'a [ShaderUniformArray],
			target: BuildTarget,
		}

		let context = OwnContext {
			api: &contents.api,
			audio_codes: &options.audio_codes,
			development: self.project.development,
			opengl_declarations: &contents.opengl_declarations,
			opengl_loading: &contents.opengl_loading,
			project_codes: &options.project_codes,
			render: &contents.render,
			settings: self.settings,
			shader_declarations: &contents.shader_declarations,
			shader_loading: &contents.shader_loading,
			shader_specific_sources: &options.shader_set.specific_sources,
			shader_uniform_arrays: &options.shader_set.uniform_arrays,
			target: build_options.target,
		};
		let contents = self
			.tera
			.render(
				"template",
				&Context::from_serialize(&context).map_err(|err| err.to_string())?,
			)
			.map_err(|err| err.to_string())?;

		let build_directory = BUILD_ROOT_DIRECTORY
			.join("executable-compilers")
			.join("crinkler");
		fs::create_dir_all(&build_directory).map_err(|err| err.to_string())?;

		let source_path = build_directory.join("executable.cpp");
		fs::write(&source_path, contents.as_bytes()).map_err(|_| "Failed to write to file.")?;

		let mut compilation = self
			.msvc_command_generator
			.command(cpp_utils::msvc::Platform::X86)
			.arg("cl")
			.arg("/c")
			.args(&self.settings.cl.args)
			.arg("/FA")
			.arg("/Faexecutable.asm")
			.arg("/Foexecutable.obj")
			.arg("executable.cpp")
			.args(&options.compilation.cl.args)
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
			.args(&options.compilation.crinkler.args)
			.arg("executable.obj")
			.current_dir(&build_directory)
			.spawn()
			.map_err(|err| err.to_string())?;

		let status = compilation.wait().map_err(|err| err.to_string())?;
		if !status.success() {
			return Err("Failed to compile.".to_string());
		}

		fs::copy(&build_directory.join("executable.exe"), &build_cache_path)
			.map_err(|err| err.to_string())?;

		Ok(build_cache_path)
	}
}

impl<'a> ExecutableCompiler for CrinklerCompiler<'a> {}

impl FileConsumer for CrinklerCompiler<'_> {
	fn get_is_path_handled<'b, 'a: 'b>(&'a self) -> IsPathHandled<'b> {
		Box::new(|path| {
			if let Some(extension) = path.extension() {
				if extension.to_string_lossy() == "cpp" {
					return true;
				}
			}
			false
		})
	}
}
