use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct MsvcSettings {
	#[serde(default)]
	pub args: Option<Vec<String>>,
}
