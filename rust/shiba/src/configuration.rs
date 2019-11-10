use crate::paths::USER_SETTINGS_DIRECTORY;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Paths {
	pub glew: Option<PathBuf>,
	pub shader_minifier: Option<PathBuf>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Configuration {
	pub paths: Paths,
}

pub fn load() -> Result<Configuration, String> {
	let path = USER_SETTINGS_DIRECTORY.join("config.yml");

	if !path.exists() {
		return Ok(Configuration::default());
	}

	let contents =
		fs::read_to_string(path).map_err(|_| "Failed to read config file.".to_string())?;
	let configuration: Configuration =
		serde_yaml::from_str(&contents).map_err(|err| format!("Failed to parse: {}.", err))?;
	Ok(configuration)
}
