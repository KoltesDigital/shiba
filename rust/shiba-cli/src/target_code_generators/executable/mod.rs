use super::api::{APIGenerator, APIGeneratorInputs, API};
use super::{GenerateTargetCodeOptions, TargetCodeGenerator};
use crate::build::{BuildOptions, BuildTarget};
use crate::compilation::{CompilationJobEmitter, Platform};
use crate::compilation_data::{Compilation, CompilationJob, CompilationJobKind};
use crate::hash_extra;
use crate::msvc;
use crate::paths::BUILD_ROOT_DIRECTORY;
use crate::project_data::Project;
use crate::project_files::CodeMap;
use crate::settings::RuntimeSettings;
use crate::shader_data::{ShaderProgramMap, ShaderUniformArray};
use crate::{Error, Result};
use serde::Serialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};

pub struct ExecutableTargetCodeGenerator {
	api_generator: APIGenerator,
	glew_path: PathBuf,
	msvc_command_generator: msvc::CommandGenerator,
	tera: Tera,
}

impl ExecutableTargetCodeGenerator {
	pub fn new(project: &Project) -> Result<Self> {
		let api_generator = APIGenerator::new(&project.configuration)?;
		let glew_path = project.configuration.get_path("glew");
		let msvc_command_generator = msvc::CommandGenerator::new()?;

		let mut tera = Tera::default();

		tera.add_raw_template("executable", include_str!("./template.tera"))
			.expect("Failed to add template.");

		Ok(ExecutableTargetCodeGenerator {
			api_generator,
			glew_path,
			msvc_command_generator,
			tera,
		})
	}
}

impl TargetCodeGenerator for ExecutableTargetCodeGenerator {
	fn generate(
		&self,
		build_options: &BuildOptions,
		options: &GenerateTargetCodeOptions,
		compilation: &mut Compilation,
	) -> Result<()> {
		const OUTPUT_FILENAME: &str = "executable.cpp";

		#[derive(Hash)]
		struct Inputs<'a> {
			api_generator: APIGeneratorInputs<'a>,
			development: bool,
			glew_path: &'a Path,
			msvc_command_generator: msvc::CommandGeneratorInputs<'a>,
			options: &'a GenerateTargetCodeOptions<'a>,
			runtime_settings: &'a RuntimeSettings,
			target: BuildTarget,
		}

		let inputs = Inputs {
			api_generator: self.api_generator.get_inputs(),
			development: build_options.project.development,
			glew_path: &self.glew_path,
			msvc_command_generator: self.msvc_command_generator.get_inputs(),
			options,
			runtime_settings: &build_options.project.settings.runtime,
			target: build_options.target,
		};
		let build_cache_directory = hash_extra::get_build_cache_directory(&inputs)?;
		let build_cache_path = build_cache_directory.join(OUTPUT_FILENAME);

		compilation
			.common
			.link_dependencies
			.insert(PathBuf::from("opengl32.lib"));

		if build_options.project.development {
			compilation.common.link_dependencies.insert(
				self.glew_path
					.join("lib")
					.join("Release")
					.join(match options.platform {
						Platform::X64 => "x64",
						Platform::X86 => "Win32",
					})
					.join("glew32s.lib"),
			);
		}

		let mut include_paths = BTreeSet::new();
		if build_options.project.development {
			include_paths.insert(self.glew_path.join("include"));
		}

		compilation.jobs.push(CompilationJob {
			kind: CompilationJobKind::Cpp,
			path: build_cache_path.clone(),
			include_paths,
		});

		if !build_options.force && build_cache_path.exists() {
			return Ok(());
		}

		let api = self.api_generator.generate(
			options.project_codes,
			options.shader_set,
			build_options.project.development,
			BuildTarget::Executable,
		)?;

		#[derive(Serialize)]
		struct OwnContext<'a> {
			api: &'a API,
			audio_codes: &'a CodeMap,
			development: bool,
			project_codes: &'a CodeMap,
			runtime_settings: &'a RuntimeSettings,
			shader_programs: &'a ShaderProgramMap,
			shader_uniform_arrays: &'a [ShaderUniformArray],
			target: BuildTarget,
		}

		let context = OwnContext {
			api: &api,
			audio_codes: &options.audio_codes,
			development: build_options.project.development,
			project_codes: &options.project_codes,
			runtime_settings: &build_options.project.settings.runtime,
			shader_programs: &options.shader_set.programs,
			shader_uniform_arrays: &options.shader_set.uniform_arrays,
			target: build_options.target,
		};
		let contents = self
			.tera
			.render(
				"executable",
				&Context::from_serialize(&context).expect("Failed to create context."),
			)
			.map_err(|err| Error::failed_to_render_template("executable", err))?;

		let build_directory = BUILD_ROOT_DIRECTORY
			.join("target-code-generators")
			.join("executable");
		fs::create_dir_all(&build_directory)
			.map_err(|err| Error::failed_to_create_directory(&build_directory, err))?;

		let source_path = build_directory.join("executable.cpp");
		fs::write(&source_path, contents.as_bytes())
			.map_err(|err| Error::failed_to_write(&source_path, err))?;

		let copy_from = build_directory.join("executable.cpp");
		fs::copy(&copy_from, &build_cache_path)
			.map_err(|err| Error::failed_to_copy(&copy_from, &build_cache_path, err))?;

		Ok(())
	}
}

impl CompilationJobEmitter for ExecutableTargetCodeGenerator {
	fn requires_asm_compiler(&self) -> bool {
		false
	}

	fn requires_cpp_compiler(&self) -> bool {
		true
	}
}
