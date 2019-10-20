use crate::directories;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigLink {
	pub args: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigPaths {
	pub glew: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
	pub link: ConfigLink,
	pub paths: ConfigPaths,
}

impl Config {
	pub fn load() -> Result<Self, cfg::ConfigError> {
		let mut config = cfg::Config::new();

		config.set_default(
			"link.args",
			vec!["/MACHINE:X64", "gdi32.lib", "opengl32.lib", "user32.lib"],
		)?;

		let config_path = (*directories::USER_SETTINGS).join("config.yml");
		let _ = config.merge(cfg::File::from(config_path));

		let _ = config.merge(cfg::File::with_name("config.yml"));

		config.try_into()
	}
}
