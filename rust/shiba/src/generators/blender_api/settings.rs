use serde::Deserialize;
use std::hash::Hash;

fn default_args() -> Vec<String> {
	vec!["/MACHINE:X64", "gdi32.lib", "opengl32.lib", "user32.lib"]
		.into_iter()
		.map(|s| s.to_string())
		.collect()
}

#[derive(Debug, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub struct Link {
	#[serde(default = "default_args")]
	pub args: Vec<String>,
}

impl Default for Link {
	fn default() -> Link {
		Link {
			args: default_args(),
		}
	}
}

#[derive(Debug, Default, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub struct Settings {
	#[serde(default)]
	pub link: Link,
}
