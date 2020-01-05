use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::PathBuf;

fn default_args() -> Vec<String> {
	vec!["/GR-", "/GS-", "/O1", "/Oi", "/Oy", "/QIfist", "/fp:fast"]
		.into_iter()
		.map(|s| s.to_string())
		.collect()
}

#[derive(Debug, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub struct CrinklerSettings {
	#[serde(default = "default_args")]
	pub args: Vec<String>,
	#[serde(default)]
	pub dependencies: BTreeSet<String>,
	#[serde(default)]
	pub library_paths: BTreeSet<PathBuf>,
}

impl Default for CrinklerSettings {
	fn default() -> Self {
		CrinklerSettings {
			dependencies: BTreeSet::default(),
			args: default_args(),
			library_paths: BTreeSet::default(),
		}
	}
}
