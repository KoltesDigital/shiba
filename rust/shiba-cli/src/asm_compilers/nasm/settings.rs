use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct NasmSettings {
	#[serde(default)]
	pub args: Vec<String>,
}
