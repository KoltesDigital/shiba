mod settings;

pub use self::settings::CrinklerSettings;
use super::{LinkOptions, Linker};
use crate::build::{BuildOptions, BuildTarget};
use crate::compilation::{Platform, PlatformDependent};
use crate::hash_extra;
use crate::msvc;
use crate::paths::BUILD_ROOT_DIRECTORY;
use crate::project_data::Project;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub struct CrinklerLinker<'a> {
	settings: &'a CrinklerSettings,

	crinkler_path: PathBuf,
	msvc_command_generator: msvc::CommandGenerator,
}

impl<'a> CrinklerLinker<'a> {
	pub fn new(project: &'a Project, settings: &'a CrinklerSettings) -> Result<Self, String> {
		let crinkler_path = project
			.configuration
			.paths
			.get("crinkler")
			.ok_or("Please set configuration key paths.crinkler.")?
			.clone();
		let msvc_command_generator = msvc::CommandGenerator::new()?;

		Ok(CrinklerLinker {
			settings,

			crinkler_path,
			msvc_command_generator,
		})
	}
}

impl<'a> Linker for CrinklerLinker<'a> {
	fn link(&self, build_options: &BuildOptions, options: &LinkOptions) -> Result<PathBuf, String> {
		assert_eq!(build_options.target, BuildTarget::Executable);

		const OUTPUT_FILENAME: &str = "crinkler.exe";

		#[derive(Hash)]
		struct Inputs<'a> {
			crinkler_path: &'a Path,
			msvc_command_generator: msvc::CommandGeneratorInputs<'a>,
			options: &'a LinkOptions<'a>,
			settings: &'a CrinklerSettings,
		}

		let inputs = Inputs {
			crinkler_path: &self.crinkler_path,
			msvc_command_generator: self.msvc_command_generator.get_inputs(),
			options,
			settings: self.settings,
		};
		let build_cache_directory = hash_extra::get_build_cache_directory(&inputs)?;
		let build_cache_path = build_cache_directory.join(OUTPUT_FILENAME);

		if !build_options.force && build_cache_path.exists() {
			return Ok(build_cache_path);
		}

		let build_directory = BUILD_ROOT_DIRECTORY.join("linkers").join("crinkler");
		fs::create_dir_all(&build_directory).map_err(|err| err.to_string())?;

		let dependencies = &options.linking.common.link_dependencies;
		let library_paths = &options.linking.common.link_library_paths;

		let mut command = self.msvc_command_generator.command(options.platform);
		command
			.arg(&self.crinkler_path)
			.args(vec![
				"/ENTRY:main",
				"/OUT:crinkler.exe",
				"/REPORT:report.html",
				"gdi32.lib",
				"kernel32.lib",
				"user32.lib",
			])
			.args(&self.settings.args)
			.args(
				self.settings
					.library_paths
					.iter()
					.map(|path| format!("/LIBPATH:{}", path.to_string_lossy())),
			)
			.args(&self.settings.dependencies)
			.args(
				library_paths
					.iter()
					.map(|path| format!("/LIBPATH:{}", path.to_string_lossy())),
			)
			.args(dependencies)
			.args(
				options
					.linking
					.sources
					.iter()
					.map(|source| source.to_string_lossy().to_string()),
			)
			.current_dir(&build_directory);

		let mut linking = command.spawn().map_err(|err| err.to_string())?;

		let status = linking.wait().map_err(|err| err.to_string())?;
		if !status.success() {
			return Err("Failed to link.".to_string());
		}

		fs::copy(&build_directory.join(OUTPUT_FILENAME), &build_cache_path)
			.map_err(|err| err.to_string())?;

		Ok(build_cache_path)
	}
}

impl PlatformDependent for CrinklerLinker<'_> {
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
