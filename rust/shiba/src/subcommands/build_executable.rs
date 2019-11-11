use crate::configuration;
use crate::custom_codes;
use crate::generators;
use crate::paths::LOCAL_DATA_DIRECTORY;
use crate::settings;
use crate::shader_minifiers;
use crate::shader_providers;
use crate::stored_hash::StoredHash;
use crate::traits::{Generator, ShaderMinifier, ShaderProvider};
use crate::types::ProjectDescriptor;
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

	let custom_codes = custom_codes::load(options.project_directory)?;

	let build_hash_path = LOCAL_DATA_DIRECTORY.join("build.hash");
	let mut build_hash = StoredHash::new(&build_hash_path);

	let cpp_hash_path = LOCAL_DATA_DIRECTORY.join("cpp.hash");
	let mut cpp_hash = StoredHash::new(&cpp_hash_path);

	let glsl_hash_path = LOCAL_DATA_DIRECTORY.join("glsl.hash");
	let mut glsl_hash = StoredHash::new(&glsl_hash_path);

	{
		let mut updater = build_hash.get_updater();
		updater.add(&project_descriptor);
		updater.add(&custom_codes);
		updater.add(&shader_provider);

		let mut updater = cpp_hash.get_updater();
		updater.add(&project_descriptor);
		updater.add(&custom_codes);

		let mut updater = glsl_hash.get_updater();
		updater.add(&shader_provider);
	}

	let must_build = options.force || build_hash.has_changed() || !generator.get_path().exists();
	let result = if must_build {
		let mut shader_descriptor = shader_provider.provide()?;

		if let Some(shader_minifier) = shader_minifier {
			shader_descriptor = shader_minifier.minify(&shader_descriptor)?;
		}

		generator.generate(&custom_codes, &shader_descriptor)?;

		let _ = build_hash.store();
		let _ = cpp_hash.store();
		let _ = glsl_hash.store();

		ResultKind::ExecutableAvailable
	} else {
		ResultKind::Nothing
	};

	let duration = start.elapsed();
	println!("Build duration: {:?}", duration);

	Ok(result)
}
