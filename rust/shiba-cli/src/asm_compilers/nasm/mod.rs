mod settings;

pub use self::settings::NasmSettings;
use crate::build::BuildOptions;
use crate::compilation::{Platform, PlatformDependent};
use crate::compilation_data::Linking;
use crate::compilers::{CompileOptions, Compiler};
use crate::hash_extra;
use crate::paths::BUILD_ROOT_DIRECTORY;
use crate::project_data::Project;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub struct NasmCompiler<'a> {
	settings: &'a NasmSettings,

	nasm_path: PathBuf,
}

impl<'a> NasmCompiler<'a> {
	pub fn new(project: &'a Project, settings: &'a NasmSettings) -> Result<Self, String> {
		let nasm_path = project
			.configuration
			.paths
			.get("nasm")
			.cloned()
			.unwrap_or_else(|| PathBuf::from("nasm"));

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
	) -> Result<(), String> {
		let output_filename = format!(
			"{}.obj",
			options
				.path
				.file_name()
				.ok_or("Invalid filename.")?
				.to_string_lossy()
		);

		let contents = fs::read_to_string(options.path).map_err(|err| err.to_string())?;

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
		fs::create_dir_all(&build_directory).map_err(|err| err.to_string())?;

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

		let mut compilation = command.spawn().map_err(|err| err.to_string())?;

		let status = compilation.wait().map_err(|err| err.to_string())?;
		if !status.success() {
			return Err("Failed to compile.".to_string());
		}

		fs::copy(&build_directory.join("file.obj"), &build_cache_path)
			.map_err(|err| err.to_string())?;

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
