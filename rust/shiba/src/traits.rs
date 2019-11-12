use crate::custom_codes::CustomCodes;
use crate::types::{CompilationDescriptor, ShaderDescriptor};
use std::hash::Hash;
use std::path::PathBuf;

pub trait AudioSynthesizer {
	fn integrate(
		&self,
		custom_codes: &mut CustomCodes,
		compilation_descriptor: &mut CompilationDescriptor,
	) -> Result<(), String>;
}

pub trait Generator {
	fn generate(
		&self,
		compilation_descriptor: &CompilationDescriptor,
		custom_codes: &CustomCodes,
		shader_descriptor: &ShaderDescriptor,
	) -> Result<(), String>;

	fn get_path(&self) -> PathBuf;
}

pub trait ShaderMinifier {
	fn minify(&self, shader_descriptor: &ShaderDescriptor) -> Result<ShaderDescriptor, String>;
}

pub trait ShaderProvider: Hash {
	fn provide(&self) -> Result<ShaderDescriptor, String>;
}
