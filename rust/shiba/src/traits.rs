use crate::types::ShaderDescriptor;
use std::hash::Hash;

pub trait ShaderMinifier {
	fn minify(&self, shader_descriptor: &ShaderDescriptor) -> Result<ShaderDescriptor, String>;
}

pub trait ShaderProvider: Hash {
	fn provide(&self) -> Result<ShaderDescriptor, String>;
}
