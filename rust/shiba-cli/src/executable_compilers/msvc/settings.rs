use crate::settings_utils::Resolution;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

fn default_link_args() -> Vec<String> {
	vec!["/MACHINE:X64", "gdi32.lib", "opengl32.lib", "user32.lib"]
		.into_iter()
		.map(|s| s.to_string())
		.collect()
}

#[derive(Debug, Deserialize, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Link {
	#[serde(default = "default_link_args")]
	pub args: Vec<String>,
}

impl Default for Link {
	fn default() -> Self {
		Link {
			args: default_link_args(),
		}
	}
}

#[derive(Debug, Default, Deserialize, Hash, Serialize)]
#[serde(rename_all(deserialize = "kebab-case", serialize = "snake_case"))]
pub struct MsvcSettings {
	#[serde(default)]
	pub close_when_finished: bool,
	#[serde(default)]
	pub duration: Option<OrderedFloat<f32>>,
	#[serde(default)]
	pub link: Link,
	#[serde(default)]
	pub loading_black_screen: bool,
	#[serde(default)]
	pub resolution: Resolution,
}
