use crate::directories;
use serde::Deserialize;

#[derive(Debug)]
pub struct ConfigProvider {
	config: cfg::Config,
}

impl ConfigProvider {
	pub fn load() -> Self {
		let mut config = cfg::Config::new();

		config
			.set_default(
				"link.args",
				vec!["/MACHINE:X64", "gdi32.lib", "opengl32.lib", "user32.lib"],
			)
			.expect("Failed to set defaults");

		let config_path = (*directories::USER_SETTINGS).join("config.yml");
		let _ = config.merge(cfg::File::from(config_path));

		let _ = config.merge(cfg::File::with_name("config.yml"));

		ConfigProvider { config }
	}

	pub fn try_into<'de, T: Deserialize<'de>>(&self) -> Result<T, String> {
		self.config
			.clone()
			.try_into()
			.map_err(|err| err.to_string())
	}
}
