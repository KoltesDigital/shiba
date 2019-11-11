use crate::generators;
use crate::shader_providers;
use serde::Deserialize;
use std::fs;
use std::hash::Hash;
use std::path::Path;

#[derive(Debug, Deserialize, Hash)]
#[serde(rename_all = "kebab-case", tag = "tool")]
pub enum Generator {
	Executable(generators::executable::Settings),
	Crinkler(generators::crinkler::Settings),
}

impl Default for Generator {
	fn default() -> Generator {
		Generator::Executable(generators::executable::Settings::default())
	}
}

#[derive(Debug, Deserialize, Hash)]
#[serde(rename_all = "kebab-case", tag = "tool")]
pub enum ShaderMinifier {
	ShaderMinifier,
}

#[derive(Debug, Deserialize, Hash)]
#[serde(rename_all = "kebab-case", tag = "tool")]
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
	#[serde(default)]
	pub generator: Generator,
	pub name: String,
	#[serde(default)]
	pub shader_minifier: Option<ShaderMinifier>,
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
