use super::{shiba, ShaderProvider};
use crate::project_data::Project;
use serde::Deserialize;

#[derive(Debug, Deserialize, Hash)]
#[serde(rename_all = "kebab-case", tag = "tool")]
pub enum Settings {
	Shiba(shiba::ShibaSettings),
}

impl Default for Settings {
	fn default() -> Self {
		Settings::Shiba(shiba::ShibaSettings::default())
	}
}

impl Settings {
	pub fn instantiate<'a>(
		&'a self,
		project: &'a Project,
	) -> Result<Box<(dyn ShaderProvider + 'a)>, String> {
		let instance: Box<(dyn ShaderProvider + 'a)> = match self {
			Settings::Shiba(settings) => {
				Box::new(shiba::ShibaShaderProvider::new(project, settings)?)
			}
		};
		Ok(instance)
	}
}
