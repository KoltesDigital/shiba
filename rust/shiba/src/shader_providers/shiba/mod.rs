mod parsers;
mod types;

use self::types::*;
use crate::config_provider::ConfigProvider;
use crate::traits;
use crate::types::{Pass, ProjectDescriptor, ShaderDescriptor, UniformArray, VariableKind};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::fs;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct ShaderProviderConfig {
	pub filename: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Config {
	pub shader_provider: ShaderProviderConfig,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
struct DefaultShaderProviderConfig {
	pub filename: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
struct DefaultConfig {
	pub shader_provider: DefaultShaderProviderConfig,
}

pub struct ShaderProvider {
	config: Config,
}

impl ShaderProvider {
	pub fn new(config_provider: &mut ConfigProvider) -> Result<Self, String> {
		let config = config_provider.get_default(DefaultConfig {
			shader_provider: DefaultShaderProviderConfig {
				filename: "shader.frag",
			},
		})?;
		Ok(ShaderProvider { config })
	}
}

fn ensure_passes_has_index(shader_descriptor: &mut ShaderDescriptor, index: usize) {
	if shader_descriptor.passes.len() <= index {
		shader_descriptor
			.passes
			.resize_with(index + 1, Pass::default);
	}
}

fn parse(code: &str) -> Result<ShaderDescriptor, String> {
	let (input, contents) = parsers::contents(code).map_err(|_| "Parsing error.".to_string())?;

	let mut shader_descriptor = ShaderDescriptor {
		glsl_version: contents.0.map(|s| s.to_owned()),
		..Default::default()
	};

	let mut prolog_code = None;
	let next_section = Cell::from(Section::Prolog);

	let mut process_code = |code| {
		lazy_static! {
			static ref IFDEF_RE: Regex =
				Regex::new(r"(?s)#ifdef\s+BUILD_ONLY(.*?)(?:#else.*?)?#endif").expect("Bad regex.");
			static ref IFNDEF_RE: Regex =
				Regex::new(r"(?s)#ifndef\s+BUILD_ONLY.*?(?:#else(.*?))?#endif")
					.expect("Bad regex.");
			static ref MAIN_RE: Regex = Regex::new(r"void main\w+\(\)").expect("Bad regex.");
		}

		let code = IFDEF_RE.replace_all(code, "$1");
		let code = IFNDEF_RE.replace_all(&code, "$1");
		let code = MAIN_RE.replace_all(&code, "void main()");

		if !code.trim().is_empty() {
			let code = Some(code.to_string());

			match next_section.get() {
				Section::Attributes => shader_descriptor.sections.attributes = code,

				Section::Common => shader_descriptor.sections.common = code,

				Section::Fragment(index) => {
					ensure_passes_has_index(&mut shader_descriptor, index);
					shader_descriptor.passes[index].fragment = code;
				}

				Section::Outputs => shader_descriptor.sections.outputs = code,

				Section::Prolog => prolog_code = code,

				Section::Varyings => shader_descriptor.sections.varyings = code,

				Section::Vertex(index) => {
					ensure_passes_has_index(&mut shader_descriptor, index);
					shader_descriptor.passes[index].vertex = code;
				}
			}
		}
	};

	for (code, section) in contents.1 {
		process_code(code);
		next_section.set(section);
	}

	process_code(input);

	if let Some(prolog_code) = &prolog_code {
		let (_, variables) =
			parsers::variables(prolog_code).map_err(|_| "Parsing error.".to_string())?;
		shader_descriptor.variables = variables;
	}

	Ok(shader_descriptor)
}

impl traits::ShaderProvider for ShaderProvider {
	fn provide_shader(
		&self,
		project_descriptor: &ProjectDescriptor,
	) -> Result<ShaderDescriptor, String> {
		let shader_contents = fs::read_to_string(
			project_descriptor
				.directory
				.join(&self.config.shader_provider.filename),
		)
		.map_err(|err| err.to_string())?;

		let mut shader_descriptor = parse(&shader_contents)?;

		if shader_descriptor.passes.is_empty() {
			return Err("Shader should define at least one pass.".to_string());
		}

		// Replace constants by their value.
		// Deactivate unreferenced variables.
		for variable in shader_descriptor.variables.iter_mut() {
			if variable.active {
				let usage_re =
					Regex::new(format!(r"\b{}\b", variable.name).as_str()).expect("Bad regex.");

				if let VariableKind::Const(value) = &variable.kind {
					/*
					console.log(
						`Replacing references to constant "${variable.name}" by its value "${variable.value}".`
					);
					*/

					let replace = |code: &String| {
						Some(
							usage_re
								.replace_all(code.as_str(), value.as_str())
								.to_string(),
						)
					};

					shader_descriptor.sections.common =
						shader_descriptor.sections.common.as_ref().and_then(replace);

					for pass in shader_descriptor.passes.iter_mut() {
						pass.vertex = pass.vertex.as_ref().and_then(replace);
						pass.fragment = pass.fragment.as_ref().and_then(replace);
					}

					variable.active = false;
				} else {
					let mut referenced = false;

					let mut find = |opt: &Option<String>| {
						if let Some(code) = opt {
							if usage_re.is_match(code) {
								referenced = true;
							}
						}
					};

					find(&shader_descriptor.sections.common);

					for pass in &shader_descriptor.passes {
						find(&pass.vertex);
						find(&pass.fragment);
					}

					if !referenced {
						/*
						console.log(
							`Global variable "${variable.name}" is not referenced and won't be used.`
						);
						*/

						variable.active = false;
					}
				}
			}
		}

		for variable in &shader_descriptor.variables {
			if !variable.active {
				continue;
			}

			if let VariableKind::Uniform = variable.kind {
				if !shader_descriptor
					.uniform_arrays
					.contains_key(&variable.type_name)
				{
					let _ = shader_descriptor.uniform_arrays.insert(
						variable.type_name.clone(),
						UniformArray {
							name: format!("_shiba_{}_uniforms", variable.type_name),
							minified_name: None,
							variables: Vec::new(),
						},
					);
				}

				let variables = &mut shader_descriptor
					.uniform_arrays
					.get_mut(&variable.type_name)
					.unwrap()
					.variables;
				variables.push(variable.clone());
			}
		}

		for uniform_array in &shader_descriptor.uniform_arrays {
			for variable in uniform_array.1.variables.iter().enumerate() {
				let usage_re =
					Regex::new(format!(r"\b{}\b", variable.1.name).as_str()).expect("Bad regex.");
				let replacement = format!("{}[{}]", uniform_array.1.name, variable.0);

				let replace = |code: &String| {
					Some(
						usage_re
							.replace_all(code.as_str(), replacement.as_str())
							.to_string(),
					)
				};

				shader_descriptor.sections.common =
					shader_descriptor.sections.common.as_ref().and_then(replace);

				for pass in shader_descriptor.passes.iter_mut() {
					pass.vertex = pass.vertex.as_ref().and_then(replace);
					pass.fragment = pass.fragment.as_ref().and_then(replace);
				}
			}
		}

		Ok(shader_descriptor)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::types::{Sections, Variable, VariableKind};

	#[test]
	fn test_parse() {
		let shader_descriptor = parse(
			r#"#version 450
float regularVar;
#define foo bar
const float constVar = 42.;
uniform float uniformVar0;
uniform float uniformVar1;
uniform vec2 uniformVar2;
#pragma shiba common
common code
#pragma shiba vertex 0
vertex code
#pragma shiba fragment 0
fragment code
"#,
		)
		.unwrap();

		assert_eq!(
			shader_descriptor,
			ShaderDescriptor {
				glsl_version: Some("450".to_string()),
				passes: vec![Pass {
					vertex: Some("vertex code\n".to_string()),
					fragment: Some("fragment code\n".to_string()),
				}],
				sections: Sections {
					common: Some("common code\n".to_string()),
					..Default::default()
				},
				variables: vec![
					Variable {
						active: true,
						kind: VariableKind::Regular,
						minified_name: None,
						name: "regularVar".to_string(),
						type_name: "float".to_string(),
					},
					Variable {
						active: true,
						kind: VariableKind::Const("42.".to_string()),
						minified_name: None,
						name: "constVar".to_string(),
						type_name: "float".to_string(),
					},
					Variable {
						active: true,
						kind: VariableKind::Uniform,
						minified_name: None,
						name: "uniformVar0".to_string(),
						type_name: "float".to_string(),
					},
					Variable {
						active: true,
						kind: VariableKind::Uniform,
						minified_name: None,
						name: "uniformVar1".to_string(),
						type_name: "float".to_string(),
					},
					Variable {
						active: true,
						kind: VariableKind::Uniform,
						minified_name: None,
						name: "uniformVar2".to_string(),
						type_name: "vec2".to_string(),
					}
				],
				..Default::default()
			}
		);
	}
}
