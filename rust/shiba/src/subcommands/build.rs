use crate::configuration;
use crate::custom_codes;
use crate::generators;
use crate::paths::LOCAL_DATA_DIRECTORY;
use crate::settings;
use crate::shader_providers;
use crate::template::TemplateRenderer;
use crate::traits::ShaderProvider;
use crate::types::ProjectDescriptor;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::time::Instant;

lazy_static! {
	pub static ref HASH_PATH: PathBuf = { LOCAL_DATA_DIRECTORY.join("build-hash") };
}

fn load_hash() -> Result<u64, String> {
	let bytes = fs::read(&*HASH_PATH).map_err(|_| "Failed to read file.")?;
	let mut cursor = Cursor::new(bytes);
	let hash = cursor.read_u64::<BigEndian>().unwrap();
	Ok(hash)
}

fn store_hash(hash: u64) -> Result<(), String> {
	let mut bytes = vec![];
	bytes.write_u64::<BigEndian>(hash).unwrap();
	fs::write(&*HASH_PATH, bytes).map_err(|_| "Failed to write to file.")?;
	Ok(())
}

pub fn subcommand(project_directory: &Path) -> Result<PathBuf, String> {
	let start = Instant::now();

	let previous_hash = load_hash().ok();

	let configuration = configuration::load()?;

	let project_descriptor = ProjectDescriptor::load(project_directory)?;

	let shader_provider = match &project_descriptor.settings.shader_provider {
		settings::ShaderProvider::Shiba(settings) => {
			shader_providers::shiba::ShaderProvider::new(project_directory, settings)
		}
	}?;

	let custom_codes = custom_codes::load(project_directory)?;

	let mut hasher = DefaultHasher::new();
	project_descriptor.hash(&mut hasher);
	shader_provider.hash(&mut hasher);
	custom_codes.hash(&mut hasher);
	let hash = hasher.finish();

	store_hash(hash)?;

	let path = generators::blender_api::BlenderAPIGenerator::get_path();

	if Some(hash) != previous_hash || !path.exists() {
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
