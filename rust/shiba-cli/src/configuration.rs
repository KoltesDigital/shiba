use crate::paths::USER_SETTINGS_DIRECTORY;
use crate::{Error, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub type ConfigurationPaths = HashMap<String, PathBuf>;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Configuration {
	paths: ConfigurationPaths,
}

impl<'a> Configuration {
	pub fn load() -> Result<Self> {
		let path = USER_SETTINGS_DIRECTORY.join("config.yml");

		if !path.exists() {
			return Ok(Configuration::default());
		}

		let contents =
			fs::read_to_string(&path).map_err(|err| Error::failed_to_read(&path, err))?;
		let configuration: Configuration = serde_yaml::from_str(&contents)
			.map_err(|err| Error::failed_to_deserialize(&contents, err))?;
		Ok(configuration)
	}

	pub fn get_path(&'a self, name: &'a str) -> PathBuf {
		match self.paths.get(name) {
			Some(path) => path.clone(),
			None => PathBuf::from(name),
		}
	}
}
