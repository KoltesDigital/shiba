pub mod settings;
pub mod shiba;

use crate::build::BuildOptions;
use crate::project_files::FileConsumer;
use crate::shader_data::ShaderSet;
pub use settings::Settings;

pub trait ShaderProvider: FileConsumer {
	fn provide(&self, build_options: &BuildOptions) -> Result<ShaderSet, String>;
}
