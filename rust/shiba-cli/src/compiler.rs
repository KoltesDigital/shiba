use crate::build::BuildOptions;
use crate::executable_compilers::ExecutableCompiler;
use crate::library_compilers::LibraryCompiler;
use crate::project_files::CodeMap;
use crate::project_files::FileConsumer;
use crate::types::{CompilationDescriptor, ShaderDescriptor};
use std::path::PathBuf;

#[derive(Hash)]
pub struct CompileOptions<'a> {
	pub audio_codes: &'a CodeMap,
	pub compilation_descriptor: &'a CompilationDescriptor,
	pub project_codes: &'a CodeMap,
	pub shader_descriptor: &'a ShaderDescriptor,
}

pub trait Compiler: FileConsumer {
	fn compile(
		&self,
		build_options: &BuildOptions,
		options: &CompileOptions,
	) -> Result<PathBuf, String>;
}

pub enum CompilerKind<'a> {
	Executable(Box<dyn ExecutableCompiler + 'a>),
	Library(Box<dyn LibraryCompiler + 'a>),
}
