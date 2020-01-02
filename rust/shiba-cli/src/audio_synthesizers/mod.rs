pub mod none;
pub mod oidos;
mod settings;

use crate::build::BuildOptions;
use crate::project_files::CodeMap;
use crate::project_files::FileConsumer;
use crate::types::CompilationDescriptor;
use serde::{Deserialize, Serialize};
pub use settings::Settings;

#[derive(Deserialize, Serialize)]
pub struct IntegrationResult {
	pub codes: CodeMap,
	pub compilation_descriptor: CompilationDescriptor,
}

pub trait AudioSynthesizer: FileConsumer {
	fn integrate(
		&self,
		build_options: &BuildOptions,
		compilation_descriptor: &CompilationDescriptor,
	) -> Result<IntegrationResult, String>;
}
