use crate::build::{BuildOptions, BuildTarget};
use crate::compiler::CompilerKind;
use crate::configuration::Configuration;
use crate::settings::Settings;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Clone, Debug, Default, Deserialize, Hash, PartialEq, Serialize)]
pub struct Pass {
	pub fragment: Option<String>,
	pub vertex: Option<String>,
}

#[derive(Debug, Default, Deserialize, Hash, PartialEq, Serialize)]
pub struct Sections {
	pub attributes: Option<String>,
	pub common: Option<String>,
	pub outputs: Option<String>,
	pub varyings: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub struct ConstVariable {
	pub value: String,
}

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub struct UniformAnnotationControlDescriptor {
	pub control_kind: String,
	pub control_parameters: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UniformAnnotationKind {
	Control(UniformAnnotationControlDescriptor),
	Time,
}

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub struct UniformVariable {
	pub annotations: Vec<UniformAnnotationKind>,
}

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum VariableKind {
	Const(ConstVariable),
	Regular,
	Uniform(UniformVariable),
}

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub struct Variable {
	pub kind: VariableKind,

	pub active: bool,
	pub length: Option<usize>,
	pub minified_name: Option<String>,
	pub name: String,
	pub type_name: String,
}

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub struct UniformArray {
	pub name: String,
	pub minified_name: Option<String>,
	pub variables: Vec<Variable>,
	pub type_name: String,
}

#[derive(Debug, Default, Deserialize, Hash, PartialEq, Serialize)]
pub struct ShaderDescriptor {
	pub glsl_version: Option<String>,
	pub sections: Sections,
	pub passes: Vec<Pass>,

	pub uniform_arrays: Vec<UniformArray>,
	pub variables: Vec<Variable>,
}

pub struct ProjectDescriptor<'a> {
	pub build_options: &'a BuildOptions<'a>,
	pub configuration: Configuration,
	pub development: bool,
	pub settings: Settings,
}

impl<'a> ProjectDescriptor<'a> {
	pub fn load(build_options: &'a BuildOptions) -> Result<Self, String> {
		let configuration = Configuration::load()?;

		let settings = Settings::load(build_options.project_directory)?;

		let development = match settings.development {
			Some(development) => development,
			None => match build_options.target {
				BuildTarget::Executable => false,
				BuildTarget::Library => true,
			},
		};

		Ok(ProjectDescriptor {
			build_options,
			configuration,
			development,
			settings,
		})
	}

	pub fn instantiate_compiler(&self) -> Result<CompilerKind, String> {
		let compiler = match self.build_options.target {
			BuildTarget::Executable => {
				CompilerKind::Executable(self.settings.executable_compiler.instantiate(self)?)
			}
			BuildTarget::Library => {
				CompilerKind::Library(self.settings.library_compiler.instantiate(self)?)
			}
		};
		Ok(compiler)
	}
}

#[derive(Clone, Default, Deserialize, Hash, Serialize)]
pub struct ClCompilationDescriptor {
	pub args: Vec<String>,
}

#[derive(Clone, Default, Deserialize, Hash, Serialize)]
pub struct CrinklerCompilationDescriptor {
	pub args: Vec<String>,
}

#[derive(Clone, Default, Deserialize, Hash, Serialize)]
pub struct LinkCompilationDescriptor {
	pub args: Vec<String>,
}

#[derive(Clone, Default, Deserialize, Hash, Serialize)]
pub struct CompilationDescriptor {
	pub cl: ClCompilationDescriptor,
	pub crinkler: CrinklerCompilationDescriptor,
	pub link: LinkCompilationDescriptor,
}
