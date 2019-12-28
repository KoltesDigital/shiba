pub mod settings;
pub mod shader_minifier;

use crate::types::ShaderDescriptor;
pub use settings::Settings;

pub trait ShaderMinifier {
	fn minify(&self, shader_descriptor: &ShaderDescriptor) -> Result<ShaderDescriptor, String>;
}
