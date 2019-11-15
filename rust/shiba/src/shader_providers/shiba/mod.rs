mod parsers;
mod settings;
mod types;

pub use self::settings::Settings;
use self::types::*;
use crate::parsers::glsl;
use crate::traits;
use crate::types::{Pass, ShaderDescriptor, UniformArray, VariableKind};
use regex::Regex;
use std::cell::Cell;
use std::fs;
use std::hash::Hash;
use std::path::Path;

#[derive(Hash)]
pub struct ShaderProvider {
	shader_contents: String,
}

impl ShaderProvider {
	pub fn new(project_directory: &Path, settings: &Settings) -> Result<Self, String> {
		let path = project_directory.join(&settings.filename);
		let shader_contents = fs::read_to_string(&path).map_err(|err| {
			format!(
				"Failed to read shader at {}: {}",
				path.to_string_lossy(),
				err
			)
		})?;

		Ok(ShaderProvider { shader_contents })
	}
}

fn ensure_passes_has_index(shader_descriptor: &mut ShaderDescriptor, index: usize) {
	if shader_descriptor.passes.len() <= index {
		shader_descriptor
			.passes
			.resize_with(index + 1, Pass::default);
	}
}

fn parse(code: &str, development: bool) -> Result<ShaderDescriptor, String> {
	let (input, (glsl_version, sections)) =
		parsers::contents(code).map_err(|_| "Parsing error.".to_string())?;

	let mut shader_descriptor = ShaderDescriptor {
		glsl_version: glsl_version.map(|s| s.to_owned()),
		..Default::default()
	};

	let mut prolog_code = None;
	let next_append_enable = Cell::from(true);
	let next_section = Cell::from(Directive::Prolog);

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
		let code = code.trim();
		if !code.is_empty() {
			let append = |section: &mut Option<String>| {
				if next_append_enable.get() {
					section.get_or_insert(String::new()).push_str(code);
				}
			};

			match next_section.get() {
				Directive::Attributes => append(&mut shader_descriptor.sections.attributes),

				Directive::Common => append(&mut shader_descriptor.sections.common),

				Directive::Fragment(index) => {
					ensure_passes_has_index(&mut shader_descriptor, index);
					append(&mut shader_descriptor.passes[index].fragment);
				}

				Directive::Outputs => append(&mut shader_descriptor.sections.outputs),

				Directive::Prolog => append(&mut prolog_code),

				Directive::Varyings => append(&mut shader_descriptor.sections.varyings),

				Directive::Vertex(index) => {
					ensure_passes_has_index(&mut shader_descriptor, index);
					append(&mut shader_descriptor.passes[index].vertex);
				}

				_ => unreachable!(),
			}
		}
	};

	let process_directive = |directive| match directive {
		Directive::Always => next_append_enable.set(true),
		Directive::Development => next_append_enable.set(development),
		directive => next_section.set(directive),
	};

	for (code, directive) in sections {
		process_code(code);
		process_directive(directive);
	}

	process_code(input);

	if let Some(prolog_code) = &prolog_code {
		let (_, variables) =
			glsl::variables(prolog_code).map_err(|_| "Parsing error.".to_string())?;
		shader_descriptor.variables = variables;
	}

	Ok(shader_descriptor)
}

impl traits::ShaderProvider for ShaderProvider {
	fn provide(&self, development: bool) -> Result<ShaderDescriptor, String> {
		let mut shader_descriptor = parse(&self.shader_contents, development)?;

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
				let uniform_array = match shader_descriptor
					.uniform_arrays
					.iter_mut()
					.find(|uniform_array| uniform_array.type_name == variable.type_name)
				{
					Some(uniform_array) => uniform_array,
					None => {
						shader_descriptor.uniform_arrays.push(UniformArray {
							name: format!("_shiba_{}_uniforms", variable.type_name),
							minified_name: None,
							variables: Vec::new(),
							type_name: variable.type_name.clone(),
						});
						shader_descriptor.uniform_arrays.last_mut().unwrap()
					}
				};
				uniform_array.variables.push(variable.clone());
			}
		}

		for uniform_array in &shader_descriptor.uniform_arrays {
			for (index, variable) in uniform_array.variables.iter().enumerate() {
				let usage_re =
					Regex::new(format!(r"\b{}\b", variable.name).as_str()).expect("Bad regex.");
				let replacement = format!("{}[{}]", uniform_array.name, index);

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
float regularVar0;
float regularVar1[1];
#define foo bar
const float constVar = 42.;
uniform float uniformVar0;
uniform float uniformVar1[4];
uniform vec2 uniformVar2;
#pragma shiba common
common code
#pragma shiba vertex 0
vertex code
#pragma shiba fragment 0
fragment code
"#,
			false,
		)
		.unwrap();

		assert_eq!(
			shader_descriptor,
			ShaderDescriptor {
				glsl_version: Some("450".to_string()),
				passes: vec![Pass {
					vertex: Some("vertex code".to_string()),
					fragment: Some("fragment code".to_string()),
				}],
				sections: Sections {
					common: Some("common code".to_string()),
					..Default::default()
				},
				variables: vec![
					Variable {
						active: true,
						kind: VariableKind::Regular,
						length: None,
						minified_name: None,
						name: "regularVar0".to_string(),
						type_name: "float".to_string(),
					},
					Variable {
						active: true,
						kind: VariableKind::Regular,
						length: Some(1),
						minified_name: None,
						name: "regularVar1".to_string(),
						type_name: "float".to_string(),
					},
					Variable {
						active: true,
						kind: VariableKind::Const("42.".to_string()),
						length: None,
						minified_name: None,
						name: "constVar".to_string(),
						type_name: "float".to_string(),
					},
					Variable {
						active: true,
						kind: VariableKind::Uniform,
						length: None,
						minified_name: None,
						name: "uniformVar0".to_string(),
						type_name: "float".to_string(),
					},
					Variable {
						active: true,
						kind: VariableKind::Uniform,
						length: Some(4),
						minified_name: None,
						name: "uniformVar1".to_string(),
						type_name: "float".to_string(),
					},
					Variable {
						active: true,
						kind: VariableKind::Uniform,
						length: None,
						minified_name: None,
						name: "uniformVar2".to_string(),
						type_name: "vec2".to_string(),
					}
				],
				..Default::default()
			}
		);
	}

	#[test]
	fn test_parse_development_false() {
		let shader_descriptor = parse(
			r#"#version 450
float first;
#pragma shiba development
float second;
#pragma shiba always
float third;
"#,
			false,
		)
		.unwrap();

		assert_eq!(
			shader_descriptor,
			ShaderDescriptor {
				glsl_version: Some("450".to_string()),
				passes: vec![],
				sections: Sections::default(),
				variables: vec![
					Variable {
						active: true,
						kind: VariableKind::Regular,
						length: None,
						minified_name: None,
						name: "first".to_string(),
						type_name: "float".to_string(),
					},
					Variable {
						active: true,
						kind: VariableKind::Regular,
						length: None,
						minified_name: None,
						name: "third".to_string(),
						type_name: "float".to_string(),
					},
				],
				..Default::default()
			}
		);
	}

	#[test]
	fn test_parse_development_true() {
		let shader_descriptor = parse(
			r#"#version 450
float first;
#pragma shiba development
float second;
#pragma shiba always
float third;
"#,
			true,
		)
		.unwrap();

		assert_eq!(
			shader_descriptor,
			ShaderDescriptor {
				glsl_version: Some("450".to_string()),
				passes: vec![],
				sections: Sections::default(),
				variables: vec![
					Variable {
						active: true,
						kind: VariableKind::Regular,
						length: None,
						minified_name: None,
						name: "first".to_string(),
						type_name: "float".to_string(),
					},
					Variable {
						active: true,
						kind: VariableKind::Regular,
						length: None,
						minified_name: None,
						name: "second".to_string(),
						type_name: "float".to_string(),
					},
					Variable {
						active: true,
						kind: VariableKind::Regular,
						length: None,
						minified_name: None,
						name: "third".to_string(),
						type_name: "float".to_string(),
					},
				],
				..Default::default()
			}
		);
	}
}
