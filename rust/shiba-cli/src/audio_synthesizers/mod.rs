pub mod none;
pub mod oidos;
mod settings;

use crate::code_map::CodeMap;
use crate::types::CompilationDescriptor;
use serde::{Deserialize, Serialize};
pub use settings::Settings;

#[derive(Deserialize, Serialize)]
pub struct IntegrationResult {
	pub codes: CodeMap,
	pub compilation_descriptor: CompilationDescriptor,
}

pub trait AudioSynthesizer {
	fn integrate(
		&self,
		compilation_descriptor: &CompilationDescriptor,
	) -> Result<IntegrationResult, String>;
}
