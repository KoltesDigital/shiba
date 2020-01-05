mod settings;

pub use self::settings::MsvcSettings;
use super::{LinkOptions, Linker};
use crate::build::{BuildOptions, BuildTarget};
use crate::compilation::{Platform, PlatformDependent};
use crate::hash_extra;
use crate::msvc;
use crate::paths::BUILD_ROOT_DIRECTORY;
use crate::project_data::Project;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

pub struct MsvcLinker<'a> {
	settings: &'a MsvcSettings,

	msvc_command_generator: msvc::CommandGenerator,
}

impl<'a> MsvcLinker<'a> {
	pub fn new(_project: &'a Project, settings: &'a MsvcSettings) -> Result<Self, String> {
		let msvc_command_generator = msvc::CommandGenerator::new()?;

		Ok(MsvcLinker {
			settings,

			msvc_command_generator,
		})
	}
}

impl<'a> Linker for MsvcLinker<'a> {
	fn link(&self, build_options: &BuildOptions, options: &LinkOptions) -> Result<PathBuf, String> {
		let output_filename = match build_options.target {
			BuildTarget::Executable => "msvc.exe",
			BuildTarget::Library => "msvc.dll",
		};

		#[derive(Hash)]
		struct Inputs<'a> {
			msvc_command_generator: msvc::CommandGeneratorInputs<'a>,
			options: &'a LinkOptions<'a>,
			settings: &'a MsvcSettings,
			target: BuildTarget,
		}

		let inputs = Inputs {
			msvc_command_generator: self.msvc_command_generator.get_inputs(),
			options,
			settings: self.settings,
			target: build_options.target,
		};
		let build_cache_directory = hash_extra::get_build_cache_directory(&inputs)?;
		let build_cache_path = build_cache_directory.join(output_filename);

		if !build_options.force && build_cache_path.exists() {
			return Ok(build_cache_path);
		}

		let build_directory = BUILD_ROOT_DIRECTORY.join("linkers").join("msvc");
		fs::create_dir_all(&build_directory).map_err(|err| err.to_string())?;

		let dependencies = &options.linking.common.link_dependencies;
		let library_paths = &options.linking.common.link_library_paths;

		let mut command = self.msvc_command_generator.command(options.platform);
		command.arg("link");
		if build_options.target == BuildTarget::Library {
			command.arg("/DLL");
		}
		command
			.arg(format!("/OUT:{}", output_filename))
			.arg(match options.platform {
				Platform::X64 => "/MACHINE:X64",
				Platform::X86 => "/MACHINE:X86",
			})
			.arg("/SUBSYSTEM:CONSOLE")
			.args(vec!["gdi32.lib", "user32.lib"])
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

		fs::copy(&build_directory.join(output_filename), &build_cache_path)
			.map_err(|err| err.to_string())?;

		Ok(build_cache_path)
	}
}

impl PlatformDependent for MsvcLinker<'_> {
	fn get_possible_platforms(&self) -> &'static BTreeSet<Platform> {
		lazy_static! {
			pub static ref POSSIBLE_PLATFORMS: BTreeSet<Platform> = {
				let mut possible_platforms = BTreeSet::new();
				possible_platforms.insert(Platform::X64);
				possible_platforms.insert(Platform::X86);
				possible_platforms
			};
		}

		&*POSSIBLE_PLATFORMS
	}
}
