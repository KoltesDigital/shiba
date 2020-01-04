use super::{msvc, LibraryCompiler};
use crate::project_data::Project;
use serde::Deserialize;

#[derive(Debug, Deserialize, Hash)]
#[serde(rename_all = "kebab-case", tag = "tool")]
pub enum Settings {
	Msvc(msvc::MsvcSettings),
}

impl Settings {
	pub fn instantiate<'a>(
		&'a self,
		project: &'a Project,
	) -> Result<Box<(dyn LibraryCompiler + 'a)>, String> {
		let instance: Box<(dyn LibraryCompiler + 'a)> = match self {
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
