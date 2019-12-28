pub mod settings;
pub mod shiba;

use crate::types::ShaderDescriptor;
pub use settings::Settings;

pub trait ShaderProvider {
	fn provide(&self) -> Result<ShaderDescriptor, String>;
}
