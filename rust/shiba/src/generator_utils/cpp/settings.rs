use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Debug, Default, Deserialize, Hash, Serialize)]
pub struct Resolution {
	pub width: Option<u32>,
	pub height: Option<u32>,
	pub scale: Option<OrderedFloat<f32>>,
}
