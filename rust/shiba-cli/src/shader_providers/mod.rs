pub mod settings;
pub mod shiba;

use crate::build::BuildOptions;
use crate::types::ShaderDescriptor;
pub use settings::Settings;

pub trait ShaderProvider {
	fn provide(&self, build_options: &BuildOptions) -> Result<ShaderDescriptor, String>;
}
