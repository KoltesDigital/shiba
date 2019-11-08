use crate::configuration;
use crate::custom_codes;
use crate::generators;
use crate::paths::LOCAL_DATA_DIRECTORY;
use crate::settings;
use crate::shader_codes::ShaderCodes;
use crate::shader_providers;
use crate::stored_hash::StoredHash;
use crate::template::TemplateRenderer;
use crate::traits::ShaderProvider;
use crate::types::{Pass, ProjectDescriptor};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::Instant;

pub struct Options<'a> {
	pub may_build_shaders_only: bool,
	pub project_directory: &'a Path,
}

#[derive(Serialize)]
pub struct ShadersAvailableDescriptor {
	pub passes: Vec<Pass>,
	pub shader_codes: ShaderCodes,
}

pub enum ResultKind {
	BlenderAPIAvailable(PathBuf),
	ShadersAvailable(ShadersAvailableDescriptor),
}

pub fn subcommand(options: &Options) -> Result<ResultKind, String> {
	let start = Instant::now();

	let configuration = configuration::load()?;

	let project_descriptor = ProjectDescriptor::load(options.project_directory)?;

	let shader_provider = match &project_descriptor.settings.shader_provider {
		settings::ShaderProvider::Shiba(settings) => {
			shader_providers::shiba::ShaderProvider::new(options.project_directory, settings)
		}
	}?;

	let custom_codes = custom_codes::load(options.project_directory)?;

	let cpp_hash_path = LOCAL_DATA_DIRECTORY.join("cpp.hash");
	let mut cpp_hash = StoredHash::new(&cpp_hash_path);

	let glsl_hash_path = LOCAL_DATA_DIRECTORY.join("glsl.hash");
	let mut glsl_hash = StoredHash::new(&glsl_hash_path);

	{
		let mut updater = cpp_hash.get_updater();
		updater.add(&project_descriptor);
		updater.add(&custom_codes);

		let mut updater = glsl_hash.get_updater();
		updater.add(&shader_provider);
	}

	let result =
		if options.may_build_shaders_only && glsl_hash.has_changed() && !cpp_hash.has_changed() {
			let shader_descriptor = shader_provider.provide()?;

			let shader_codes = ShaderCodes::load(&shader_descriptor);

			let _ = glsl_hash.store();

			Ok(ResultKind::ShadersAvailable(ShadersAvailableDescriptor {
				passes: shader_descriptor.passes,
				shader_codes,
			}))
		} else {
			let path = generators::blender_api::BlenderAPIGenerator::get_path();

			if glsl_hash.has_changed() || cpp_hash.has_changed() || !path.exists() {
				let generator = generators::blender_api::BlenderAPIGenerator::new(
					&project_descriptor.settings.blender_api,
					&configuration,
				)?;

				let shader_descriptor = shader_provider.provide()?;

				let template_renderer = TemplateRenderer::new()?;

				generator.generate(
					options.project_directory,
					&template_renderer,
					&shader_descriptor,
				)?;

				let _ = cpp_hash.store();
				let _ = glsl_hash.store();
			}

			Ok(ResultKind::BlenderAPIAvailable(path))
		};

	let duration = start.elapsed();
	println!("Build duration: {:?}", duration);

	result
}
