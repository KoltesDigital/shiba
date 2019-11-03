use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Default, PartialEq, Serialize)]
pub struct Pass {
	pub fragment: Option<String>,
	pub vertex: Option<String>,
}

#[derive(Debug, Default, PartialEq, Serialize)]
pub struct Sections {
	pub attributes: Option<String>,
	pub common: Option<String>,
	pub outputs: Option<String>,
	pub varyings: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum VariableKind {
	Const(String),
	Regular,
	Uniform,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Variable {
	pub kind: VariableKind,

	pub active: bool,
	pub minified_name: Option<String>,
	pub name: String,
	pub type_name: String,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct UniformArray {
	pub name: String,
	pub minified_name: Option<String>,
	pub variables: Vec<Variable>,
}

#[derive(Debug, Default, PartialEq)]
pub struct ShaderDescriptor {
	pub glsl_version: Option<String>,
	pub sections: Sections,
	pub passes: Vec<Pass>,

	pub uniform_arrays: HashMap<String, UniformArray>,
	pub variables: Vec<Variable>,
}

#[derive(Debug)]
pub struct ProjectDescriptor {
	pub directory: PathBuf,
}
