use serde::Deserialize;
use std::path::PathBuf;

fn default_path() -> PathBuf {
	PathBuf::from("music.xrns")
}

#[derive(Debug, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub struct OidosSettings {
	#[serde(default = "default_path")]
	pub path: PathBuf,
}

impl Default for OidosSettings {
	fn default() -> Self {
		OidosSettings {
			path: default_path(),
		}
	}
}
