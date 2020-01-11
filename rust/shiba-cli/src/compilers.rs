use crate::build::BuildOptions;
use crate::compilation::{Platform, PlatformDependent};
use crate::compilation_data::Linking;
use crate::project_files::CodeMap;
use crate::shader_data::ShaderSet;
use crate::Result;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Hash)]
pub struct CompileOptions<'a> {
	pub audio_codes: &'a CodeMap,
	pub include_paths: &'a BTreeSet<PathBuf>,
	pub path: &'a Path,
	pub platform: Platform,
	pub project_codes: &'a CodeMap,
	pub shader_set: &'a ShaderSet,
}

pub trait Compiler: PlatformDependent {
	fn compile(
		&self,
		build_options: &BuildOptions,
		options: &CompileOptions,
		linking: &mut Linking,
	) -> Result<()>;
}
