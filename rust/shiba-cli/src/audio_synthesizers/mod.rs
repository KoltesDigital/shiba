pub mod none;
pub mod oidos;
mod settings;

use crate::build::BuildOptions;
use crate::compilation_data::Compilation;
use crate::project_files::CodeMap;
use crate::project_files::FileConsumer;
use crate::compilation::CompilationJobEmitter;
pub use settings::Settings;

pub trait AudioSynthesizer: FileConsumer + CompilationJobEmitter {
	fn integrate(
		&self,
		build_options: &BuildOptions,
		compilation: &mut Compilation,
	) -> Result<CodeMap, String>;
}
