mod settings;

pub use self::settings::MsvcSettings;
use super::LibraryCompiler;
use crate::build::{BuildOptions, BuildTarget};
use crate::compiler::{CompileOptions, Compiler};
use crate::cpp_utils;
use crate::hash_extra;
use crate::paths::BUILD_ROOT_DIRECTORY;
use crate::project_data::Project;
use crate::project_files::{CodeMap, FileConsumer, IsPathHandled};
use crate::shader_data::ShaderSet;
use serde::Serialize;
use std::fs;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};

pub struct MsvcCompiler<'a> {
	project: &'a Project,
	settings: &'a MsvcSettings,

	api_generator: cpp_utils::api::Generator,
	glew_path: PathBuf,
	msvc_command_generator: cpp_utils::msvc::CommandGenerator,
	tera: Tera,
}

impl<'a> MsvcCompiler<'a> {
	pub fn new(project: &'a Project, settings: &'a MsvcSettings) -> Result<Self, String> {
		let api_generator = cpp_utils::api::Generator::new(&project.configuration)?;
		let glew_path = project
			.configuration
			.paths
			.get("glew")
			.ok_or("Please set configuration key paths.glew.")?
			.clone();
		let msvc_command_generator = cpp_utils::msvc::CommandGenerator::new()?;

		let mut tera = Tera::default();

		tera.add_raw_template("template", include_str!("template.tera"))
			.map_err(|err| err.to_string())?;

		Ok(MsvcCompiler {
			project,
			settings,

			api_generator,
			glew_path,
			msvc_command_generator,
			tera,
		})
	}
}

impl<'a> Compiler for MsvcCompiler<'a> {
	fn compile(
		&self,
		build_options: &BuildOptions,
		options: &CompileOptions,
	) -> Result<PathBuf, String> {
		const OUTPUT_FILENAME: &str = "msvc.dll";

		#[derive(Hash)]
		struct Inputs<'a> {
			api_generator: cpp_utils::api::GeneratorInputs<'a>,
			development: bool,
			glew_path: &'a Path,
			msvc_command_generator: cpp_utils::msvc::CommandGeneratorInputs<'a>,
			options: &'a CompileOptions<'a>,
			settings: &'a MsvcSettings,
			target: BuildTarget,
		}

		let inputs = Inputs {
			api_generator: self.api_generator.get_inputs(),
			development: self.project.development,
			glew_path: &self.glew_path,
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

		let api = self.api_generator.generate(
			options.project_codes,
			options.shader_set,
			self.project.development,
			BuildTarget::Library,
		)?;

		#[derive(Serialize)]
		struct OwnContext<'a> {
			api: &'a cpp_utils::api::Contents,
			project_codes: &'a CodeMap,
			shader_set: &'a ShaderSet,
			shader_specific_sources_length: usize,
		}

		let context = OwnContext {
			api: &api,
			project_codes: &options.project_codes,
			shader_set: &options.shader_set,
			shader_specific_sources_length: options.shader_set.specific_sources.len(),
		};
		let contents = self
			.tera
			.render(
				"template",
				&Context::from_serialize(&context).map_err(|err| err.to_string())?,
			)
			.map_err(|err| err.to_string())?;

		let build_directory = BUILD_ROOT_DIRECTORY.join("library-compilers").join("msvc");
		fs::create_dir_all(&build_directory).map_err(|err| err.to_string())?;

		let source_path = build_directory.join("library.cpp");
		fs::write(&source_path, contents.as_bytes()).map_err(|_| "Failed to write to file.")?;

		let mut compilation = self
			.msvc_command_generator
			.command(cpp_utils::msvc::Platform::X64)
			.arg("cl")
			.arg("/c")
			.arg("/EHsc")
			.arg("/FA")
			.arg("/Falibrary.asm")
			.arg("/Folibrary.obj")
			.arg(format!(
				"/I{}",
				self.glew_path.join("include").to_string_lossy(),
			))
			.args(&options.compilation.cl.args)
			.arg("library.cpp")
			.arg("&&")
			.arg("link")
			.arg("/DLL")
			.arg("/OUT:library.dll")
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
			.args(&options.compilation.link.args)
			.arg("library.obj")
			.current_dir(&build_directory)
			.spawn()
			.map_err(|err| err.to_string())?;

		let status = compilation.wait().map_err(|err| err.to_string())?;
		if !status.success() {
			return Err("Failed to compile.".to_string());
		}

		fs::copy(&build_directory.join("library.dll"), &build_cache_path)
			.map_err(|err| err.to_string())?;

		Ok(build_cache_path)
	}
}

impl<'a> LibraryCompiler for MsvcCompiler<'a> {}

impl FileConsumer for MsvcCompiler<'_> {
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
