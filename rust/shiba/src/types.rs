use crate::settings::{self, Settings};
use serde::Serialize;
use std::collections::BTreeMap;
use std::hash::Hash;
use std::path::Path;

#[derive(Debug, Default, Hash, PartialEq, Serialize)]
pub struct Pass {
	pub fragment: Option<String>,
	pub vertex: Option<String>,
}

#[derive(Debug, Default, Hash, PartialEq, Serialize)]
pub struct Sections {
	pub attributes: Option<String>,
	pub common: Option<String>,
	pub outputs: Option<String>,
	pub varyings: Option<String>,
}

#[derive(Clone, Debug, Hash, PartialEq, Serialize)]
pub enum VariableKind {
	Const(String),
	Regular,
	Uniform,
}

#[derive(Clone, Debug, Hash, PartialEq, Serialize)]
pub struct Variable {
	pub kind: VariableKind,

	pub active: bool,
	pub minified_name: Option<String>,
	pub name: String,
	pub type_name: String,
}

#[derive(Debug, Hash, PartialEq, Serialize)]
pub struct UniformArray {
	pub name: String,
	pub minified_name: Option<String>,
	pub variables: Vec<Variable>,
}

#[derive(Debug, Default, Hash, PartialEq)]
pub struct ShaderDescriptor {
	pub glsl_version: Option<String>,
	pub sections: Sections,
	pub passes: Vec<Pass>,

	pub uniform_arrays: BTreeMap<String, UniformArray>,
	pub variables: Vec<Variable>,
}

#[derive(Debug, Hash)]
pub struct ProjectDescriptor {
	pub settings: Settings,
}

impl ProjectDescriptor {
	pub fn load(project_directory: &Path) -> Result<Self, String> {
		let settings = settings::load(project_directory)?;
		Ok(ProjectDescriptor { settings })
	}
}
