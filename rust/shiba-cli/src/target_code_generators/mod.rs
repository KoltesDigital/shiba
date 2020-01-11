mod api;
pub mod executable;
pub mod library;

use crate::build::BuildOptions;
use crate::compilation::{CompilationJobEmitter, Platform};
use crate::compilation_data::Compilation;
use crate::project_files::CodeMap;
use crate::shader_data::ShaderSet;
use crate::Result;

#[derive(Hash)]
pub struct GenerateTargetCodeOptions<'a> {
	pub audio_codes: &'a CodeMap,
	pub platform: Platform,
	pub project_codes: &'a CodeMap,
	pub shader_set: &'a ShaderSet,
}

pub trait TargetCodeGenerator: CompilationJobEmitter {
	fn generate(
		&self,
		build_options: &BuildOptions,
		options: &GenerateTargetCodeOptions,
		compilation: &mut Compilation,
	) -> Result<()>;
}
