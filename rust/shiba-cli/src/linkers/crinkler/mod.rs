mod settings;

pub use self::settings::CrinklerSettings;
use super::{LinkOptions, Linker};
use crate::build::{BuildOptions, BuildTarget};
use crate::compilation::{Platform, PlatformDependent};
use crate::hash_extra;
use crate::msvc;
use crate::paths::BUILD_ROOT_DIRECTORY;
use crate::project_data::Project;
use crate::{Error, Result};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub struct CrinklerLinker<'a> {
	settings: &'a CrinklerSettings,

	crinkler_path: PathBuf,
	msvc_command_generator: msvc::CommandGenerator,
}

impl<'a> CrinklerLinker<'a> {
	pub fn new(project: &'a Project, settings: &'a CrinklerSettings) -> Result<Self> {
		let crinkler_path = project.configuration.get_path("crinkler");
		let msvc_command_generator = msvc::CommandGenerator::new()?;

		Ok(CrinklerLinker {
			settings,

			crinkler_path,
			msvc_command_generator,
		})
	}
}

impl<'a> Linker for CrinklerLinker<'a> {
	fn link(&self, build_options: &BuildOptions, options: &LinkOptions) -> Result<PathBuf> {
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
		fs::create_dir_all(&build_directory)
			.map_err(|err| Error::failed_to_create_directory(&build_directory, err))?;

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

		let mut linking = command
			.spawn()
			.map_err(|err| Error::failed_to_execute(&self.crinkler_path, err))?;

		let status = linking.wait().unwrap();
		if !status.success() {
			return Err(Error::execution_failed(&self.crinkler_path));
		}

		let copy_from = build_directory.join(OUTPUT_FILENAME);
		fs::copy(&copy_from, &build_cache_path)
			.map_err(|err| Error::failed_to_copy(&copy_from, &build_cache_path, err))?;

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
