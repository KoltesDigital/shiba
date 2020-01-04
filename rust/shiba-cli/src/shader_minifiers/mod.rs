pub mod settings;
pub mod shader_minifier;

use crate::build::BuildOptions;
use crate::shader_data::ShaderSet;
pub use settings::Settings;

pub trait ShaderMinifier {
	fn minify(
		&self,
		build_options: &BuildOptions,
		shader_set: &ShaderSet,
	) -> Result<ShaderSet, String>;
}
