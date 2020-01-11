mod settings;

pub use self::settings::NasmSettings;
use crate::build::BuildOptions;
use crate::compilation::{Platform, PlatformDependent};
use crate::compilation_data::Linking;
use crate::compilers::{CompileOptions, Compiler};
use crate::hash_extra;
use crate::paths::BUILD_ROOT_DIRECTORY;
use crate::project_data::Project;
use crate::{Error, Result};
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub struct NasmCompiler<'a> {
	settings: &'a NasmSettings,

	nasm_path: PathBuf,
}

impl<'a> NasmCompiler<'a> {
	pub fn new(project: &'a Project, settings: &'a NasmSettings) -> Result<Self> {
		let nasm_path = project.configuration.get_path("nasm");

		Ok(NasmCompiler {
			settings,

			nasm_path,
		})
	}
}

impl<'a> Compiler for NasmCompiler<'a> {
	fn compile(
		&self,
		build_options: &BuildOptions,
		options: &CompileOptions,
		linking: &mut Linking,
	) -> Result<()> {
		let output_filename = format!(
			"{}.obj",
			options
				.path
				.file_name()
				.ok_or_else(|| Error::path_has_invalid_file_name(options.path))?
				.to_string_lossy()
		);

		let contents = fs::read_to_string(options.path)
			.map_err(|err| Error::failed_to_read(options.path, err))?;

		#[derive(Hash)]
		struct Inputs<'a> {
			contents: &'a str,
			options: &'a CompileOptions<'a>,
			settings: &'a NasmSettings,
		}

		let inputs = Inputs {
			contents: &contents,
			options,
			settings: self.settings,
		};
		let build_cache_directory = hash_extra::get_build_cache_directory(&inputs)?;
		let build_cache_path = build_cache_directory.join(output_filename);

		linking.sources.push(build_cache_path.clone());

		if !build_options.force && build_cache_path.exists() {
			return Ok(());
		}

		let build_directory = BUILD_ROOT_DIRECTORY.join("cpp-compilers").join("nasm");
		fs::create_dir_all(&build_directory)
			.map_err(|err| Error::failed_to_create_directory(&build_directory, err))?;

		let mut command = Command::new(&self.nasm_path);
		command.args(vec!["-f", "win32"]);

		for path in options.include_paths {
			command.arg("-i").arg(path);
		}

		command
			.arg("-o")
			.arg("file.obj")
			.arg(options.path.to_string_lossy().as_ref())
			.current_dir(&build_directory);

		let mut compilation = command
			.spawn()
			.map_err(|err| Error::failed_to_execute(&self.nasm_path, err))?;

		let status = compilation.wait().unwrap();
		if !status.success() {
			return Err(Error::execution_failed(&self.nasm_path));
		}

		let copy_from = build_directory.join("file.obj");
		fs::copy(&copy_from, &build_cache_path)
			.map_err(|err| Error::failed_to_copy(&copy_from, &build_cache_path, err))?;

		Ok(())
	}
}

impl PlatformDependent for NasmCompiler<'_> {
	fn get_possible_platforms(&self) -> &'static BTreeSet<Platform> {
		lazy_static! {
			pub static ref POSSIBLE_PLATFORMS: BTreeSet<Platform> = {
				let mut possible_platforms = BTreeSet::new();
				possible_platforms.insert(Platform::X86);
				possible_platforms
			};
		}

		&*POSSIBLE_PLATFORMS
	}
}
