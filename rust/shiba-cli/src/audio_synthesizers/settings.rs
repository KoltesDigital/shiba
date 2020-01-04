use super::{none, oidos, AudioSynthesizer};
use crate::project_data::Project;
use serde::Deserialize;

#[derive(Debug, Deserialize, Hash)]
#[serde(rename_all = "kebab-case", tag = "tool")]
pub enum Settings {
	None(none::NoneSettings),
	Oidos(oidos::OidosSettings),
}

impl Settings {
	pub fn instantiate<'a>(
		&'a self,
		project: &'a Project,
	) -> Result<Box<(dyn AudioSynthesizer + 'a)>, String> {
		let instance: Box<(dyn AudioSynthesizer + 'a)> = match self {
			Settings::None(settings) => {
				Box::new(none::NoneAudioSynthesizer::new(project, settings)?)
			}
			Settings::Oidos(settings) => {
				Box::new(oidos::OidosAudioSynthesizer::new(project, settings)?)
			}
		};
		Ok(instance)
	}
}

impl Default for Settings {
	fn default() -> Self {
		Settings::None(none::NoneSettings::default())
	}
}
