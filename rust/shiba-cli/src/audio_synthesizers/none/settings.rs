use ordered_float::OrderedFloat;
use serde::Deserialize;
use std::hash::Hash;

#[derive(Debug, Default, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub struct NoneSettings {
	#[serde(default)]
	pub speed: Option<OrderedFloat<f32>>,
}
