use serde::Deserialize;
use std::hash::Hash;

fn default_filename() -> String {
	"music.xrns".to_string()
}

#[derive(Debug, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub struct OidosSettings {
	#[serde(default = "default_filename")]
	pub filename: String,
}

impl Default for OidosSettings {
	fn default() -> Self {
		OidosSettings {
			filename: default_filename(),
		}
	}
}
