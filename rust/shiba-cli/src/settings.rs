use crate::audio_synthesizers;
use crate::executable_compilers;
use crate::library_compilers;
use crate::shader_minifiers;
use crate::shader_providers;
use serde::Deserialize;
use std::fs;
use std::hash::Hash;
use std::path::Path;

#[derive(Debug, Default, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub struct Settings {
	#[serde(default)]
	pub audio_synthesizer: audio_synthesizers::Settings,
	pub development: Option<bool>,
	#[serde(default)]
	pub executable_compiler: executable_compilers::Settings,
	#[serde(default)]
	pub library_compiler: library_compilers::Settings,
	#[serde(default)]
	pub name: String,
	#[serde(default)]
	pub shader_minifier: Option<shader_minifiers::Settings>,
	#[serde(default)]
	pub shader_provider: shader_providers::Settings,
	pub shiba_version: Option<String>,
}

impl Settings {
	pub fn load(project_directory: &Path) -> Result<Self, String> {
		let path = project_directory.join("shiba.yml");

		if !path.exists() {
			return Ok(Settings::default());
		}

		let contents = fs::read_to_string(path).map_err(|_| "Failed to open settings file.")?;
		let project: Settings =
			serde_yaml::from_str(&contents).map_err(|err| format!("Failed to parse: {}.", err))?;
		Ok(project)
	}
}
