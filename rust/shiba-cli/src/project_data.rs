use crate::build::BuildTarget;
use crate::configuration::Configuration;
use crate::settings::Settings;
use crate::Result;
use std::path::{Path, PathBuf};

pub struct Project {
	pub configuration: Configuration,
	pub development: bool,
	pub directory: PathBuf,
	pub settings: Settings,
}

impl<'a> Project {
	pub fn load(directory: &'a Path, target: BuildTarget) -> Result<Self> {
		let configuration = Configuration::load()?;

		let settings = Settings::load(directory)?;

		let development = match settings.development {
			Some(development) => development,
			None => match target {
				BuildTarget::Executable => false,
				BuildTarget::Library => true,
			},
		};

		Ok(Project {
			configuration,
			development,
			directory: PathBuf::from(directory),
			settings,
		})
	}
}
