mod parsers;
mod settings;
mod types;

pub use self::settings::ShibaSettings;
use self::types::*;
use super::ShaderProvider;
use crate::build::{BuildOptions, BuildTarget};
use crate::hash_extra;
use crate::parsers::glsl;
use crate::project_data::Project;
use crate::project_files::{FileConsumer, IsPathHandled};
use crate::shader_data::{
	ShaderConstVariable, ShaderSet, ShaderSource, ShaderUniformArray, ShaderVariableKind,
};
use regex::Regex;
use serde::Serialize;
use serde_json;
use std::cell::Cell;
use std::fs;
use std::path::PathBuf;
use tera::{Context, Tera};

#[derive(Serialize)]
struct OwnContext {
	development: bool,
	target: BuildTarget,
}

pub struct ShibaShaderProvider<'a> {
	project: &'a Project,

	contents: String,
	path: PathBuf,
}

impl<'a> ShibaShaderProvider<'a> {
	pub fn new(project: &'a Project, settings: &'a ShibaSettings) -> Result<Self, String> {
		let path = project.directory.join(&settings.filename);
		let contents = fs::read_to_string(&path).map_err(|err| {
			format!(
				"Failed to read shader at {}: {}",
				path.to_string_lossy(),
				err
			)
		})?;

		Ok(ShibaShaderProvider {
			project,
			contents,
			path,
		})
	}

	fn render(&self, build_options: &BuildOptions, code: &str) -> Result<String, String> {
		let mut tera = Tera::default();

		tera.add_raw_template("shader-provider-shiba", code)
			.map_err(|err| err.to_string())?;

		let context = OwnContext {
			development: self.project.development,
			target: build_options.target,
		};

		let code = tera
			.render(
				"shader-provider-shiba",
				&Context::from_serialize(&context).map_err(|err| err.to_string())?,
			)
			.map_err(|err| err.to_string())?;

		Ok(code)
	}
}

fn get_specific_source<'a>(shader_set: &'a mut ShaderSet, name: &str) -> &'a mut ShaderSource {
	shader_set
		.specific_sources
		.entry(name.to_string())
		.or_insert_with(ShaderSource::default)
}

fn parse(code: &str) -> Result<ShaderSet, String> {
	let (input, (glsl_version, sections)) =
		parsers::contents(code).map_err(|_| "Parsing error.".to_string())?;

	let mut shader_set = ShaderSet {
		glsl_version: glsl_version.map(|s| s.to_owned()),
		..Default::default()
	};

	let mut prolog_code = None;
	let next_append_enable = Cell::from(true);
	let next_section = Cell::from(Directive::Prolog);

	let mut process_code = |code| {
		lazy_static! {
			static ref MAIN_RE: Regex =
				Regex::new(r"(?s)void\s+main\w+\s*\(\s*\)").expect("Bad regex.");
		}

		let code = MAIN_RE.replace_all(code, "void main()");
		let code = code.trim();
		if !code.is_empty() {
			let append = |section: &mut Option<String>| {
				if next_append_enable.get() {
					section.get_or_insert(String::new()).push_str(code);
				}
			};

			match next_section.get() {
				Directive::Attributes => append(&mut shader_set.sections.attributes),

				Directive::Common => append(&mut shader_set.sections.common),

				Directive::Fragment(name) => {
					let source = get_specific_source(&mut shader_set, name);
					append(&mut source.fragment);
				}

				Directive::Outputs => append(&mut shader_set.sections.outputs),

				Directive::Prolog => append(&mut prolog_code),

				Directive::Varyings => append(&mut shader_set.sections.varyings),

				Directive::Vertex(name) => {
					let source = get_specific_source(&mut shader_set, name);
					append(&mut source.vertex);
				}
			}
		}
	};

	for (code, directive) in sections {
		process_code(code);
		next_section.set(directive);
	}

	process_code(input);

	if let Some(prolog_code) = &prolog_code {
		let (_, variables) =
			glsl::variables(prolog_code).map_err(|_| "Parsing error.".to_string())?;
		shader_set.variables = variables;
	}

	Ok(shader_set)
}

impl<'a> ShaderProvider for ShibaShaderProvider<'a> {
	fn provide(&self, build_options: &BuildOptions) -> Result<ShaderSet, String> {
		const OUTPUT_FILENAME: &str = "shader-descriptor.json";

		#[derive(Hash)]
		struct Inputs<'a> {
			development: bool,
			contents: &'a String,
			target: BuildTarget,
		}

		let inputs = Inputs {
			development: self.project.development,
			contents: &self.contents,
			target: build_options.target,
		};
		let build_cache_directory = hash_extra::get_build_cache_directory(&inputs)?;
		let build_cache_path = build_cache_directory.join(OUTPUT_FILENAME);

		if !build_options.force && build_cache_path.exists() {
			let json = fs::read_to_string(build_cache_path).map_err(|err| err.to_string())?;
			let shader_set =
				serde_json::from_str(json.as_str()).map_err(|_| "Failed to parse JSON.")?;
			return Ok(shader_set);
		}

		let contents = self.render(build_options, &self.contents)?;

		let mut shader_set = parse(&contents)?;

		if shader_set.specific_sources.is_empty() {
			return Err("Shader set should define at least one shader.".to_string());
		}

		// Replace constants by their value.
		// Deactivate unreferenced variables.
		for variable in shader_set.variables.iter_mut() {
			if variable.active {
				let usage_re =
					Regex::new(format!(r"\b{}\b", variable.name).as_str()).expect("Bad regex.");

				if let ShaderVariableKind::Const(ShaderConstVariable { value }) = &variable.kind {
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

					shader_set.sections.common =
						shader_set.sections.common.as_ref().and_then(replace);

					for (_name, shader_source) in shader_set.specific_sources.iter_mut() {
						shader_source.vertex = shader_source.vertex.as_ref().and_then(replace);
						shader_source.fragment = shader_source.fragment.as_ref().and_then(replace);
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

					find(&shader_set.sections.common);

					for (_name, shader_source) in shader_set.specific_sources.iter() {
						find(&shader_source.vertex);
						find(&shader_source.fragment);
					}

					if !referenced {
						variable.active = false;
					}
				}
			}
		}

		for variable in &shader_set.variables {
			if !variable.active {
				continue;
			}

			if let ShaderVariableKind::Uniform(_) = variable.kind {
				let uniform_array = match shader_set
					.uniform_arrays
					.iter_mut()
					.find(|uniform_array| uniform_array.type_name == variable.type_name)
				{
					Some(uniform_array) => uniform_array,
					None => {
						shader_set.uniform_arrays.push(ShaderUniformArray {
							name: format!("_shiba_{}_uniforms", variable.type_name),
							minified_name: None,
							variables: Vec::new(),
							type_name: variable.type_name.clone(),
						});
						shader_set.uniform_arrays.last_mut().unwrap()
					}
				};
				uniform_array.variables.push(variable.clone());
			}
		}

		for uniform_array in &shader_set.uniform_arrays {
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

				shader_set.sections.common = shader_set.sections.common.as_ref().and_then(replace);

				for (_name, shader_source) in shader_set.specific_sources.iter_mut() {
					shader_source.vertex = shader_source.vertex.as_ref().and_then(replace);
					shader_source.fragment = shader_source.fragment.as_ref().and_then(replace);
				}
			}
		}

		let json = serde_json::to_string(&shader_set).map_err(|_| "Failed to dump JSON.")?;
		fs::write(build_cache_path, json).map_err(|err| err.to_string())?;

		Ok(shader_set)
	}
}

impl FileConsumer for ShibaShaderProvider<'_> {
	fn get_is_path_handled<'b, 'a: 'b>(&'a self) -> IsPathHandled<'b> {
		Box::new(move |path| path == self.path)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::shader_data::{
		ShaderSections, ShaderSourceMap, ShaderUniformVariable, ShaderVariable, ShaderVariableKind,
	};

	#[test]
	fn test_parse() {
		let shader_set = parse(
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
#pragma shiba vertex shader
vertex code
#pragma shiba fragment shader
fragment code
"#,
		)
		.unwrap();

		let mut expected_specific_sources = ShaderSourceMap::new();
		expected_specific_sources.insert(
			"shader".to_string(),
			ShaderSource {
				vertex: Some("vertex code".to_string()),
				fragment: Some("fragment code".to_string()),
			},
		);

		assert_eq!(
			shader_set,
			ShaderSet {
				glsl_version: Some("450".to_string()),
				sections: ShaderSections {
					common: Some("common code".to_string()),
					..Default::default()
				},
				specific_sources: expected_specific_sources,
				variables: vec![
					ShaderVariable {
						active: true,
						kind: ShaderVariableKind::Regular,
						length: None,
						minified_name: None,
						name: "regularVar0".to_string(),
						type_name: "float".to_string(),
					},
					ShaderVariable {
						active: true,
						kind: ShaderVariableKind::Regular,
						length: Some(1),
						minified_name: None,
						name: "regularVar1".to_string(),
						type_name: "float".to_string(),
					},
					ShaderVariable {
						active: true,
						kind: ShaderVariableKind::Const(ShaderConstVariable {
							value: "42.".to_string()
						}),
						length: None,
						minified_name: None,
						name: "constVar".to_string(),
						type_name: "float".to_string(),
					},
					ShaderVariable {
						active: true,
						kind: ShaderVariableKind::Uniform(ShaderUniformVariable {
							annotations: vec![]
						}),
						length: None,
						minified_name: None,
						name: "uniformVar0".to_string(),
						type_name: "float".to_string(),
					},
					ShaderVariable {
						active: true,
						kind: ShaderVariableKind::Uniform(ShaderUniformVariable {
							annotations: vec![]
						}),
						length: Some(4),
						minified_name: None,
						name: "uniformVar1".to_string(),
						type_name: "float".to_string(),
					},
					ShaderVariable {
						active: true,
						kind: ShaderVariableKind::Uniform(ShaderUniformVariable {
							annotations: vec![]
						}),
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
}
