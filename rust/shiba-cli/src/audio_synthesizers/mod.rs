pub mod none;
pub mod oidos;
mod settings;

use crate::code_map::CodeMap;
use crate::types::CompilationDescriptor;
pub use settings::Settings;

pub trait AudioSynthesizer {
	fn integrate(
		&self,
		compilation_descriptor: &mut CompilationDescriptor,
	) -> Result<CodeMap, String>;
}
