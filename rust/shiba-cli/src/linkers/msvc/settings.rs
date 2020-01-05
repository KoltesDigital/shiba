use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub struct MsvcSettings {
	#[serde(default)]
	pub args: Vec<String>,
	#[serde(default)]
	pub dependencies: BTreeSet<String>,
	#[serde(default)]
	pub library_paths: BTreeSet<PathBuf>,
}
