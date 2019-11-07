use crate::configuration;
use crate::custom_codes;
use crate::generators;
use crate::paths::LOCAL_DATA_DIRECTORY;
use crate::settings;
use crate::shader_providers;
use crate::stored_hash::StoredHash;
use crate::template::TemplateRenderer;
use crate::traits::ShaderProvider;
use crate::types::ProjectDescriptor;
use std::path::{Path, PathBuf};
use std::time::Instant;

pub fn subcommand(project_directory: &Path) -> Result<PathBuf, String> {
	let start = Instant::now();

	let build_hash_path = LOCAL_DATA_DIRECTORY.join("build.hash");
	let mut build_hash = StoredHash::new(&build_hash_path);

	let configuration = configuration::load()?;

	let project_descriptor = ProjectDescriptor::load(project_directory)?;

	let shader_provider = match &project_descriptor.settings.shader_provider {
		settings::ShaderProvider::Shiba(settings) => {
			shader_providers::shiba::ShaderProvider::new(project_directory, settings)
		}
	}?;

	let custom_codes = custom_codes::load(project_directory)?;

	{
		let mut updater = build_hash.get_updater();
		updater.add(&project_descriptor);
		updater.add(&shader_provider);
		updater.add(&custom_codes);
	}

	let path = generators::blender_api::BlenderAPIGenerator::get_path();

	if build_hash.has_changed() || !path.exists() {
		let generator = generators::blender_api::BlenderAPIGenerator::new(
			&project_descriptor.settings.blender_api,
			&configuration,
		)?;

		let shader_descriptor = shader_provider.provide()?;
		println!("{:?}", shader_descriptor);

		let template_renderer = TemplateRenderer::new()?;

		generator.generate(project_directory, &template_renderer, &shader_descriptor)?;
	}

	let duration = start.elapsed();
	println!("Time elapsed in building is: {:?}", duration);

	Ok(path)
}
