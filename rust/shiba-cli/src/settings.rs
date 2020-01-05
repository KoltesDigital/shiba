use crate::asm_compilers;
use crate::audio_synthesizers;
use crate::cpp_compilers;
use crate::executable_linkers;
use crate::library_linkers;
use crate::shader_minifiers;
use crate::shader_providers;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Default, Deserialize, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Resolution {
	pub width: Option<u32>,
	pub height: Option<u32>,
	pub scale: Option<OrderedFloat<f32>>,
}

#[derive(Debug, Default, Deserialize, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct RuntimeSettings {
	#[serde(default)]
	pub close_when_finished: bool,
	#[serde(default)]
	pub duration: Option<OrderedFloat<f32>>,
	#[serde(default)]
	pub loading_black_screen: bool,
	#[serde(default)]
	pub resolution: Resolution,
}

#[derive(Debug, Default, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub struct Settings {
	pub shiba_version: Option<String>,

	pub name: String,
	pub development: Option<bool>,
	#[serde(default)]
	pub runtime: RuntimeSettings,

	#[serde(default)]
	pub asm_compiler: asm_compilers::Settings,
	#[serde(default)]
	pub audio_synthesizer: audio_synthesizers::Settings,
	#[serde(default)]
	pub cpp_compiler: cpp_compilers::Settings,
	#[serde(default)]
	pub executable_linker: executable_linkers::Settings,
	#[serde(default)]
	pub library_linker: library_linkers::Settings,
	#[serde(default)]
	pub shader_minifier: Option<shader_minifiers::Settings>,
	#[serde(default)]
	pub shader_provider: shader_providers::Settings,
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
