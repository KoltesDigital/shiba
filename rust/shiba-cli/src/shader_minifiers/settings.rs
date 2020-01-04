use super::{shader_minifier, ShaderMinifier};
use crate::project_data::Project;
use serde::Deserialize;

#[derive(Debug, Deserialize, Hash)]
#[serde(rename_all = "kebab-case", tag = "tool")]
pub enum Settings {
	ShaderMinifier,
}

impl Settings {
	pub fn instantiate<'a>(
		&'a self,
		project: &'a Project,
	) -> Result<Box<(dyn ShaderMinifier + 'a)>, String> {
		let instance: Box<(dyn ShaderMinifier + 'a)> = match self {
			Settings::ShaderMinifier => {
				Box::new(shader_minifier::ShaderMinifierShaderMinifier::new(project)?)
			}
		};
		Ok(instance)
	}
}
