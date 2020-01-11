mod settings;

pub use self::settings::MsvcSettings;
use crate::build::BuildOptions;
use crate::compilation::{Platform, PlatformDependent};
use crate::compilation_data::Linking;
use crate::compilers::{CompileOptions, Compiler};
use crate::hash_extra;
use crate::msvc;
use crate::paths::BUILD_ROOT_DIRECTORY;
use crate::project_data::Project;
use crate::{Error, Result};
use std::collections::BTreeSet;
use std::fs;

pub struct MsvcCompiler<'a> {
	settings: &'a MsvcSettings,

	args: Vec<String>,
	msvc_command_generator: msvc::CommandGenerator,
}

impl<'a> MsvcCompiler<'a> {
	pub fn new(project: &'a Project, settings: &'a MsvcSettings) -> Result<Self> {
		let args = settings.args.clone().unwrap_or_else(|| {
			if project.development {
				vec!["/EHsc"]
			} else {
				vec![
					"/O1",
					"/Oi",
					"/Oy",
					"/GR-",
					"/GS-",
					"/fp:fast",
					"/arch:IA32",
				]
			}
			.into_iter()
			.map(|s| s.to_string())
			.collect()
		});
		let msvc_command_generator = msvc::CommandGenerator::new()?;

		Ok(MsvcCompiler {
			settings,

			args,
			msvc_command_generator,
		})
	}
}

impl<'a> Compiler for MsvcCompiler<'a> {
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
			msvc_command_generator: msvc::CommandGeneratorInputs<'a>,
			options: &'a CompileOptions<'a>,
			settings: &'a MsvcSettings,
		}

		let inputs = Inputs {
			contents: &contents,
			msvc_command_generator: self.msvc_command_generator.get_inputs(),
			options,
			settings: self.settings,
		};
		let build_cache_directory = hash_extra::get_build_cache_directory(&inputs)?;
		let build_cache_path = build_cache_directory.join(output_filename);

		linking.sources.push(build_cache_path.clone());

		if !build_options.force && build_cache_path.exists() {
			return Ok(());
		}

		let build_directory = BUILD_ROOT_DIRECTORY.join("cpp-compilers").join("msvc");
		fs::create_dir_all(&build_directory)
			.map_err(|err| Error::failed_to_create_directory(&build_directory, err))?;

		let mut compilation = self
			.msvc_command_generator
			.command(options.platform)
			.arg("cl")
			.arg("/c")
			.arg("/EHsc")
			.arg("/FA")
			.arg("/Fofile.obj")
			.args(&self.args)
			.args(
				options
					.include_paths
					.iter()
					.map(|path| format!("/I{}", path.to_string_lossy())),
			)
			.arg(options.path)
			.current_dir(&build_directory)
			.spawn()
			.map_err(|err| Error::failed_to_execute("cl", err))?;

		let status = compilation.wait().unwrap();
		if !status.success() {
			return Err(Error::execution_failed("cl"));
		}

		let copy_from = build_directory.join("file.obj");
		fs::copy(&copy_from, &build_cache_path)
			.map_err(|err| Error::failed_to_copy(&copy_from, &build_cache_path, err))?;

		Ok(())
	}
}

impl PlatformDependent for MsvcCompiler<'_> {
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
