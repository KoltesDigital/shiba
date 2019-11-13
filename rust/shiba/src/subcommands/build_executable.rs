use crate::audio_synthesizers;
use crate::code_map;
use crate::configuration;
use crate::generators;
use crate::paths::LOCAL_DATA_DIRECTORY;
use crate::settings;
use crate::shader_minifiers;
use crate::shader_providers;
use crate::stored_hash::StoredHash;
use crate::traits::{AudioSynthesizer, Generator, ShaderMinifier, ShaderProvider};
use crate::types::{CompilationDescriptor, ProjectDescriptor};
use std::fs;
use std::path::Path;
use std::time::Instant;

pub struct Options<'a> {
	pub force: bool,
	pub project_directory: &'a Path,
}

pub enum ResultKind {
	ExecutableAvailable,
	Nothing,
}

pub fn subcommand(options: &Options) -> Result<ResultKind, String> {
	let start = Instant::now();

	let configuration = configuration::load()?;

	let project_descriptor = ProjectDescriptor::load(options.project_directory)?;

	let audio_synthesizer: Box<dyn AudioSynthesizer> =
		match &project_descriptor.settings.audio_synthesizer {
			settings::AudioSynthesizer::None(settings) => {
				Box::new(audio_synthesizers::none::AudioSynthesizer::new(settings)?)
			}
			settings::AudioSynthesizer::Oidos(settings) => {
				Box::new(audio_synthesizers::oidos::AudioSynthesizer::new(
					options.project_directory,
					settings,
					&configuration,
				)?)
			}
		};

	let shader_minifier = project_descriptor
		.settings
		.shader_minifier
		.as_ref()
		.map(|shader_minifier| match shader_minifier {
			settings::ShaderMinifier::ShaderMinifier => {
				shader_minifiers::shader_minifier::ShaderMinifier::new(&configuration)
			}
		})
		.transpose()?;

	let shader_provider = match &project_descriptor.settings.shader_provider {
		settings::ShaderProvider::Shiba(settings) => {
			shader_providers::shiba::ShaderProvider::new(options.project_directory, settings)
		}
	}?;

	let generator: Box<dyn Generator> = match &project_descriptor.settings.generator {
		settings::Generator::Crinkler(settings) => Box::new(generators::crinkler::Generator::new(
			settings,
			&configuration,
		)?),
		settings::Generator::Executable(settings) => Box::new(
			generators::executable::Generator::new(settings, &configuration)?,
		),
	};

	let project_codes =
		code_map::load_project_codes(options.project_directory, generator.get_development())?;

	let build_hash_path = LOCAL_DATA_DIRECTORY.join("executable.build.hash");
	let mut build_hash = StoredHash::new(&build_hash_path);

	{
		let mut updater = build_hash.get_updater();
		updater.add(&project_descriptor);
		updater.add(&project_codes);
		updater.add(&shader_provider);
	}

	let must_build = options.force || build_hash.has_changed() || !generator.get_path().exists();
	let result = if must_build {
		let mut compilation_descriptor = CompilationDescriptor::default();

		let audio_codes = audio_synthesizer.integrate(&mut compilation_descriptor)?;

		let mut shader_descriptor = shader_provider.provide()?;

		if let Some(shader_minifier) = shader_minifier {
			shader_descriptor = shader_minifier.minify(&shader_descriptor)?;
		}

		generator.generate(
			&audio_codes,
			&compilation_descriptor,
			&project_codes,
			&shader_descriptor,
		)?;

		let _ = build_hash.store();

		ResultKind::ExecutableAvailable
	} else {
		ResultKind::Nothing
	};

	let duration = start.elapsed();
	println!("Build duration: {:?}", duration);

	Ok(result)
}

pub fn get_path(project_directory: &Path) -> Result<u64, String> {
	let configuration = configuration::load()?;

	let project_descriptor = ProjectDescriptor::load(project_directory)?;

	let generator: Box<dyn Generator> = match &project_descriptor.settings.generator {
		settings::Generator::Crinkler(settings) => Box::new(generators::crinkler::Generator::new(
			settings,
			&configuration,
		)?),
		settings::Generator::Executable(settings) => Box::new(
			generators::executable::Generator::new(settings, &configuration)?,
		),
	};

	let metadata =
		fs::metadata(&generator.get_path()).map_err(|_| "Failed to retrieve metadata.")?;

	Ok(metadata.len())
}
