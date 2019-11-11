use crate::configuration;
use crate::custom_codes;
use crate::generators;
use crate::paths::LOCAL_DATA_DIRECTORY;
use crate::settings;
use crate::shader_codes::to_standalone_passes;
use crate::shader_minifiers;
use crate::shader_providers;
use crate::stored_hash::StoredHash;
use crate::traits::{Generator, ShaderMinifier, ShaderProvider};
use crate::types::{Pass, ProjectDescriptor};
use std::path::Path;
use std::time::Instant;

pub struct Options<'a> {
	pub diff: bool,
	pub force: bool,
	pub project_directory: &'a Path,
}

pub enum ResultKind {
	BlenderAPIAvailable,
	Nothing,
	ShaderPassesAvailable(Vec<Pass>),
}

enum WhatToBuild {
	BlenderAPI,
	Nothing,
	ShaderPasses,
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

	let custom_codes = custom_codes::load(options.project_directory)?;

	let build_hash_path = LOCAL_DATA_DIRECTORY.join("build.hash");
	let mut build_hash = StoredHash::new(&build_hash_path);

	let cpp_hash_path = LOCAL_DATA_DIRECTORY.join("cpp.hash");
	let mut cpp_hash = StoredHash::new(&cpp_hash_path);

	let glsl_hash_path = LOCAL_DATA_DIRECTORY.join("glsl.hash");
	let mut glsl_hash = StoredHash::new(&glsl_hash_path);

	{
		let mut updater = build_hash.get_updater();
		updater.add(&custom_codes);
		updater.add(&project_descriptor);
		updater.add(&shader_provider);

		let mut updater = cpp_hash.get_updater();
		updater.add(&custom_codes);
		updater.add(&project_descriptor);

		let mut updater = glsl_hash.get_updater();
		updater.add(&shader_provider);
	}

	let what_to_build = if options.force || !generators::blender_api::Generator::get_path().exists()
	{
		WhatToBuild::BlenderAPI
	} else {
		if options.diff && !cpp_hash.has_changed() {
			if glsl_hash.has_changed() {
				WhatToBuild::ShaderPasses
			} else {
				WhatToBuild::Nothing
			}
		} else {
			if build_hash.has_changed() {
				WhatToBuild::BlenderAPI
			} else {
				WhatToBuild::Nothing
			}
		}
	};

	let result = match what_to_build {
		WhatToBuild::BlenderAPI => {
			let generator = generators::blender_api::Generator::new(
				&project_descriptor.settings.blender_api,
				&configuration,
			)?;

			let mut shader_descriptor = shader_provider.provide()?;

			if let Some(shader_minifier) = shader_minifier {
				shader_descriptor = shader_minifier.minify(&shader_descriptor)?;
			}

			generator.generate(&custom_codes, &shader_descriptor)?;

			let _ = build_hash.store();
			let _ = cpp_hash.store();
			let _ = glsl_hash.store();

			ResultKind::BlenderAPIAvailable
		}
		WhatToBuild::Nothing => ResultKind::Nothing,
		WhatToBuild::ShaderPasses => {
			let mut shader_descriptor = shader_provider.provide()?;

			if let Some(shader_minifier) = shader_minifier {
				shader_descriptor = shader_minifier.minify(&shader_descriptor)?;
			}

			let shader_passes = to_standalone_passes(&shader_descriptor);

			let _ = glsl_hash.store();

			ResultKind::ShaderPassesAvailable(shader_passes)
		}
	};

	let duration = start.elapsed();
	println!("Build duration: {:?}", duration);

	Ok(result)
}
