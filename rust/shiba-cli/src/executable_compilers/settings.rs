use super::{crinkler, msvc, ExecutableCompiler};
use crate::project_data::Project;
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
		project: &'a Project,
	) -> Result<Box<(dyn ExecutableCompiler + 'a)>, String> {
		let instance: Box<(dyn ExecutableCompiler + 'a)> = match self {
			Settings::Crinkler(settings) => {
				Box::new(crinkler::CrinklerCompiler::new(project, settings)?)
			}
			Settings::Msvc(settings) => Box::new(msvc::MsvcCompiler::new(project, settings)?),
		};
		Ok(instance)
	}
}

impl Default for Settings {
	fn default() -> Self {
		Settings::Msvc(msvc::MsvcSettings::default())
	}
}
