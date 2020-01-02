pub mod settings;
pub mod shader_minifier;

use crate::build::BuildOptions;
use crate::types::ShaderDescriptor;
pub use settings::Settings;

pub trait ShaderMinifier {
	fn minify(
		&self,
		build_options: &BuildOptions,
		shader_descriptor: &ShaderDescriptor,
	) -> Result<ShaderDescriptor, String>;
}
