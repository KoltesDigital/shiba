use crate::generator_utils::settings::Resolution;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

fn default_cl_args() -> Vec<String> {
	vec![
		"/GR-",
		"/GS-",
		"/O1",
		"/Oi",
		"/Oy",
		"/QIfist",
		"/fp:fast",
		"/arch:IA32",
	]
	.into_iter()
	.map(|s| s.to_string())
	.collect()
}

#[derive(Debug, Deserialize, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Cl {
	#[serde(default = "default_cl_args")]
	pub args: Vec<String>,
}

impl Default for Cl {
	fn default() -> Self {
		Cl {
			args: default_cl_args(),
		}
	}
}

#[derive(Debug, Default, Deserialize, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Crinkler {
	#[serde(default)]
	pub args: Vec<String>,
}

#[derive(Debug, Default, Deserialize, Hash, Serialize)]
#[serde(rename_all(deserialize = "kebab-case", serialize = "snake_case"))]
pub struct CrinklerSettings {
	#[serde(default)]
	pub close_when_finished: bool,
	#[serde(default)]
	pub duration: Option<OrderedFloat<f32>>,
	#[serde(default)]
	pub cl: Cl,
	#[serde(default)]
	pub crinkler: Crinkler,
	#[serde(default)]
	pub loading_black_screen: bool,
	#[serde(default)]
	pub resolution: Resolution,
}
