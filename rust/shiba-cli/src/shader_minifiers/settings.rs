use super::{shader_minifier, ShaderMinifier};
use crate::types::ProjectDescriptor;
use serde::Deserialize;

#[derive(Debug, Deserialize, Hash)]
#[serde(rename_all = "kebab-case", tag = "tool")]
pub enum Settings {
	ShaderMinifier,
}

impl Settings {
	pub fn instantiate<'a>(
		&'a self,
		project_descriptor: &'a ProjectDescriptor,
	) -> Result<Box<(dyn ShaderMinifier + 'a)>, String> {
		let instance: Box<(dyn ShaderMinifier + 'a)> = match self {
			Settings::ShaderMinifier => Box::new(
				shader_minifier::ShaderMinifierShaderMinifier::new(project_descriptor)?,
			),
		};
		Ok(instance)
	}
}
