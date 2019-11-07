use crate::types::ShaderDescriptor;
use std::hash::Hash;

pub trait ShaderProvider: Hash {
	fn provide(&self) -> Result<ShaderDescriptor, String>;
}
