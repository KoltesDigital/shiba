#[macro_use]
extern crate lazy_static;

mod config_provider;
mod directories;
mod generators {
	pub mod blender_api;
}
mod parsers;
mod shader_providers {
	pub mod shiba;
}
mod template;
mod traits;
mod types;

use crate::template::TemplateRenderer;
use crate::traits::ShaderProvider;
use crate::types::ProjectDescriptor;
use config_provider::ConfigProvider;
use serde::Deserialize;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct ShaderProviderConfig {
	pub r#type: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Config {
	pub shader_provider: ShaderProviderConfig,
}

fn main() -> Result<(), String> {
	let mut config_provider = ConfigProvider::load()?;

	let config: Config = config_provider.get()?;

	let shader_provider = match config.shader_provider.r#type.as_str() {
		"shiba" => shader_providers::shiba::ShaderProvider::new(&mut config_provider)?,
		_ => return Err("Unknown shader provider.".to_string()),
	};

	let generator = generators::blender_api::BlenderAPIGenerator::new(&config_provider)?;

	let project_descriptor = ProjectDescriptor {
		directory: PathBuf::from_str("../example").unwrap(),
	};

	let shader_descriptor = shader_provider.provide_shader(&project_descriptor)?;
	println!("{:?}", shader_descriptor);

	let template_renderer = TemplateRenderer::new()?;

	generator.generate(&template_renderer, &project_descriptor, &shader_descriptor)?;

	Ok(())
}
