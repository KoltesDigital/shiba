use super::{crinkler, msvc, ExecutableCompiler};
use crate::types::ProjectDescriptor;
use serde::Deserialize;

#[derive(Debug, Deserialize, Hash)]
#[serde(rename_all = "kebab-case", tag = "tool")]
pub enum Settings {
	Crinkler(crinkler::CrinklerSettings),
	Msvc(msvc::MsvcSettings),
}

impl Settings {
	pub fn instantiate<'a>(
		&'a self,
		project_descriptor: &'a ProjectDescriptor,
	) -> Result<Box<(dyn ExecutableCompiler + 'a)>, String> {
		let instance: Box<(dyn ExecutableCompiler + 'a)> = match self {
			Settings::Crinkler(settings) => Box::new(crinkler::CrinklerCompiler::new(
				project_descriptor,
				settings,
			)?),
			Settings::Msvc(settings) => {
				Box::new(msvc::MsvcCompiler::new(project_descriptor, settings)?)
			}
		};
		Ok(instance)
	}
}

impl Default for Settings {
	fn default() -> Self {
		Settings::Msvc(msvc::MsvcSettings::default())
	}
}
