use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Deserialize, Hash, PartialEq, Serialize)]
pub struct ShaderProgram {
	pub fragment: Option<String>,
	pub vertex: Option<String>,
}

pub type ShaderProgramMap = BTreeMap<String, ShaderProgram>;

#[derive(Debug, Default, Deserialize, Hash, PartialEq, Serialize)]
pub struct ShaderSections {
	pub attributes: Option<String>,
	pub common: Option<String>,
	pub outputs: Option<String>,
	pub varyings: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub struct ShaderConstVariable {
	pub value: String,
}

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ShaderUniformAnnotationControl {
	pub parameters: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case", tag = "kind")]
pub enum ShaderUniformAnnotationKind {
	Control(ShaderUniformAnnotationControl),
	InverseProjection,
	InverseView,
	Projection,
	ResolutionHeight,
	ResolutionWidth,
	Time,
	View,
}

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub struct ShaderUniformVariable {
	pub annotations: Vec<ShaderUniformAnnotationKind>,
}

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case", tag = "kind")]
pub enum ShaderVariableKind {
	Const(ShaderConstVariable),
	Regular,
	Uniform(ShaderUniformVariable),
}

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ShaderVariable {
	#[serde(flatten)]
	pub kind: ShaderVariableKind,

	pub active: bool,
	pub length: Option<usize>,
	pub minified_name: Option<String>,
	pub name: String,
	pub type_name: String,
}

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub struct ShaderUniformArray {
	pub name: String,
	pub minified_name: Option<String>,
	pub variables: Vec<ShaderVariable>,
	pub type_name: String,
}

#[derive(Debug, Default, Deserialize, Hash, PartialEq, Serialize)]
pub struct ShaderSet {
	pub glsl_version: Option<String>,
	pub sections: ShaderSections,
	pub programs: ShaderProgramMap,

	pub uniform_arrays: Vec<ShaderUniformArray>,
	pub variables: Vec<ShaderVariable>,
}
