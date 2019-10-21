use serde::Serialize;

#[derive(Debug, Default, Serialize)]
pub struct Pass {
	pub fragment: Option<String>,
	pub vertex: Option<String>,
}
