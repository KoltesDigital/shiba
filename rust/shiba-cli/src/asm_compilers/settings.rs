use super::nasm;
use crate::compilers::Compiler;
use crate::project_data::Project;
use serde::Deserialize;

#[derive(Debug, Deserialize, Hash)]
#[serde(rename_all = "kebab-case", tag = "tool")]
pub enum Settings {
	Nasm(nasm::NasmSettings),
}

impl Settings {
	pub fn instantiate<'a>(
		&'a self,
		project: &'a Project,
	) -> Result<Box<(dyn Compiler + 'a)>, String> {
		let instance: Box<(dyn Compiler + 'a)> = match self {
			Settings::Nasm(settings) => Box::new(nasm::NasmCompiler::new(project, settings)?),
		};
		Ok(instance)
	}
}

impl Default for Settings {
	fn default() -> Self {
		Settings::Nasm(nasm::NasmSettings::default())
	}
}
