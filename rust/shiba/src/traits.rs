use crate::code_map::CodeMap;
use crate::types::{CompilationDescriptor, ShaderDescriptor};
use std::hash::Hash;
use std::path::PathBuf;

pub trait AudioSynthesizer {
	fn integrate(
		&self,
		compilation_descriptor: &mut CompilationDescriptor,
	) -> Result<CodeMap, String>;
}

pub trait Generator {
	fn generate(
		&self,
		audio_codes: &CodeMap,
		compilation_descriptor: &CompilationDescriptor,
		project_codes: &CodeMap,
		shader_descriptor: &ShaderDescriptor,
	) -> Result<(), String>;

	fn get_development(&self) -> bool;
	fn get_path(&self) -> PathBuf;
}

pub trait ShaderMinifier {
	fn minify(&self, shader_descriptor: &ShaderDescriptor) -> Result<ShaderDescriptor, String>;
}

pub trait ShaderProvider: Hash {
	fn provide(&self) -> Result<ShaderDescriptor, String>;
}
