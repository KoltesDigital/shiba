use crate::linkers::{crinkler, msvc, Linker};
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
	) -> Result<Box<(dyn Linker + 'a)>, String> {
		let instance: Box<(dyn Linker + 'a)> = match self {
			Settings::Crinkler(settings) => {
				Box::new(crinkler::CrinklerLinker::new(project, settings)?)
			}
			Settings::Msvc(settings) => Box::new(msvc::MsvcLinker::new(project, settings)?),
		};
		Ok(instance)
	}
}

impl Default for Settings {
	fn default() -> Self {
		Settings::Msvc(msvc::MsvcSettings::default())
	}
}
