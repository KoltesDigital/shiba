use serde::Deserialize;
use std::hash::Hash;

fn default_filename() -> String {
	"shader.frag".to_string()
}

#[derive(Debug, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub struct ShibaSettings {
	#[serde(default = "default_filename")]
	pub filename: String,
}

impl Default for ShibaSettings {
	fn default() -> Self {
		ShibaSettings {
			filename: default_filename(),
		}
	}
}
