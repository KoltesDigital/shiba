pub mod none;
pub mod oidos;
mod settings;

use crate::build::BuildOptions;
use crate::compilation_data::Compilation;
use crate::project_files::CodeMap;
use crate::project_files::FileConsumer;
use serde::{Deserialize, Serialize};
pub use settings::Settings;

#[derive(Deserialize, Serialize)]
pub struct IntegrationResult {
	pub codes: CodeMap,
	pub compilation: Compilation,
}

pub trait AudioSynthesizer: FileConsumer {
	fn integrate(
		&self,
		build_options: &BuildOptions,
		compilation: &Compilation,
	) -> Result<IntegrationResult, String>;
}
