use crate::types::ShaderDescriptor;
use std::collections::BTreeMap;
use std::hash::Hash;
use std::path::PathBuf;

pub trait Generator {
	fn generate(
		&self,
		custom_codes: &BTreeMap<String, String>,
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
