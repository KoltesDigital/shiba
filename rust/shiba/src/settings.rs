use crate::generators;
use crate::shader_providers;
use ordered_float::OrderedFloat;
use serde::Deserialize;
use std::fs;
use std::hash::Hash;
use std::path::Path;

#[derive(Debug, Default, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub struct Resolution {
	pub width: Option<i32>,
	pub height: Option<i32>,
	pub scale: Option<OrderedFloat<f32>>,
}

#[derive(Debug, Deserialize, Hash)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum ShaderProvider {
	Shiba(shader_providers::shiba::Settings),
}

impl Default for ShaderProvider {
	fn default() -> ShaderProvider {
		ShaderProvider::Shiba(shader_providers::shiba::Settings::default())
	}
}

#[derive(Debug, Default, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub struct Settings {
	#[serde(default)]
	pub blender_api: generators::blender_api::Settings,
	pub name: String,
	pub resolution: Option<Resolution>,
	#[serde(default)]
	pub shader_provider: ShaderProvider,
	pub shiba: Option<String>,
}

pub fn load(project_directory: &Path) -> Result<Settings, String> {
	let path = project_directory.join("shiba.yml");

	if !path.exists() {
		return Ok(Settings::default());
	}

	let contents = fs::read_to_string(path).map_err(|_| "Failed to open settings file.")?;
	let project: Settings =
		serde_yaml::from_str(&contents).map_err(|err| format!("Failed to parse: {}.", err))?;
	Ok(project)
}
