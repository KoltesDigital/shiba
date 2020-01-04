use crate::build::BuildTarget;
use crate::configuration::Configuration;
use crate::project_files::CodeMap;
use crate::shader_codes::ShaderCodes;
use crate::shader_data::{
	ShaderSet, ShaderSource, ShaderSourceMap, ShaderUniformArray, ShaderVariable,
	ShaderVariableKind,
};
use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tera::{Context, Tera, Value};

template_enum! {
	API: "api",
	OpenGLDeclarations: "opengl_declarations",
	OpenGLLoading: "opengl_loading",
	Render: "render",
	SetActiveUniformValues: "set_active_uniform_values",
	ShaderDeclarations: "shader_declarations",
	ShaderLoading: "shader_loading",
}

#[derive(Serialize)]
pub struct Contents {
	pub api: String,
	pub opengl_declarations: String,
	pub opengl_loading: String,
	pub render: String,
	pub set_active_uniform_values: String,
	pub shader_declarations: String,
	pub shader_loading: String,
}

#[derive(Hash)]
pub struct GeneratorInputs<'a> {
	pub glew_path: &'a Option<PathBuf>,
}

pub struct Generator {
	glew_path: Option<PathBuf>,
	tera: Tera,
}

impl Generator {
	pub fn new(configuration: &Configuration) -> Result<Self, String> {
		let glew_path = configuration.paths.get("glew").cloned();

		let mut tera = Tera::default();

		tera.add_raw_templates(Template::as_array())
			.map_err(|err| err.to_string())?;

		tera.register_filter(
			"string_literal",
			|value: &Value, args: &HashMap<String, Value>| match value {
				Value::Null => match args.get("nullptr") {
					Some(s) => Ok(s.clone()),
					None => Ok(Value::String("nullptr".to_string())),
				},
				Value::String(old) => {
					let new = format!(
						"\"{}\"",
						old.replace("\n", "\\n")
							.replace("\r", "")
							.replace("\"", "\\\"")
					);
					Ok(Value::String(new))
				}
				_ => Err(tera::Error::from("string_literal expects a string")),
			},
		);

		Ok(Generator { glew_path, tera })
	}

	pub fn get_inputs(&self) -> GeneratorInputs {
		GeneratorInputs {
			glew_path: &self.glew_path,
		}
	}

	pub fn generate(
		&self,
		project_codes: &CodeMap,
		shader_set: &ShaderSet,
		development: bool,
		target: BuildTarget,
	) -> Result<Contents, String> {
		let shader_codes = ShaderCodes::load(shader_set);

		#[derive(Serialize)]
		struct ShaderUniform<'a> {
			#[serde(flatten)]
			variable: &'a ShaderVariable,

			as_value_name: &'static str,
		}

		let active_uniforms = shader_set
			.variables
			.iter()
			.filter_map(|variable| {
				if variable.active {
					if let ShaderVariableKind::Uniform(_) = &variable.kind {
						let as_value_name = to_as_value_name(variable.type_name.as_str());
						return Some(ShaderUniform {
							as_value_name,
							variable,
						});
					}
				}
				None
			})
			.collect::<Vec<_>>();

		#[derive(Serialize)]
		struct ShaderUniformArrayExt<'a> {
			#[serde(flatten)]
			uniform_array: &'a ShaderUniformArray,

			first_letter_uppercased_type_name: String,
			opengl_type_name: &'static str,
			opengl_uniform_call: String,
		}

		let shader_uniform_arrays = shader_set
			.uniform_arrays
			.iter()
			.map(|uniform_array| {
				let mut first_letter_uppercased_type_name = uniform_array.type_name.clone();
				if let Some(c) = first_letter_uppercased_type_name.get_mut(0..1) {
					c.make_ascii_uppercase();
				}

				let opengl_type_name = to_opengl_type_name(&uniform_array.type_name);
				let opengl_uniform_call = to_opengl_uniform_call(
					&uniform_array.type_name,
					&first_letter_uppercased_type_name,
				);

				ShaderUniformArrayExt {
					uniform_array,

					first_letter_uppercased_type_name,
					opengl_type_name,
					opengl_uniform_call,
				}
			})
			.collect::<Vec<_>>();

		#[derive(Serialize)]
		struct APIContext<'a> {
			development: bool,
			shader_specific_sources: &'a ShaderSourceMap,
			shader_specific_sources_length: usize,
			target: BuildTarget,
		}

		let api_context = APIContext {
			development,
			shader_specific_sources: &shader_set.specific_sources,
			shader_specific_sources_length: shader_set.specific_sources.len(),
			target,
		};
		let api = self.render_template(Template::API, &api_context)?;

		#[derive(Serialize)]
		struct RenderContext<'a> {
			project_codes: &'a CodeMap,
			shader_variables: &'a [ShaderVariable],
			target: BuildTarget,
		}

		let render_context = RenderContext {
			project_codes: &project_codes,
			shader_variables: &shader_set.variables,
			target,
		};
		let render = self.render_template(Template::Render, &render_context)?;

		#[derive(Serialize)]
		struct SetActiveUniformValuesContext<'a> {
			active_uniforms: &'a [ShaderUniform<'a>],
			target: BuildTarget,
		}

		let set_active_uniform_values_context = SetActiveUniformValuesContext {
			active_uniforms: &active_uniforms,
			target,
		};
		let set_active_uniform_values = self.render_template(
			Template::SetActiveUniformValues,
			&set_active_uniform_values_context,
		)?;

		let (_, shader_specific_source_0) = &shader_set.specific_sources.iter().next().unwrap();

		#[derive(Serialize)]
		struct ShaderDeclarationContext<'a> {
			active_uniforms: &'a [ShaderUniform<'a>],
			shader_codes: &'a ShaderCodes,
			shader_specific_sources: &'a ShaderSourceMap,
			shader_specific_sources_length: usize,
			shader_uniform_arrays: &'a [ShaderUniformArrayExt<'a>],
			target: BuildTarget,
		}

		let shader_declarations_context = ShaderDeclarationContext {
			active_uniforms: &active_uniforms,
			shader_codes: &shader_codes,
			shader_specific_sources: &shader_set.specific_sources,
			shader_specific_sources_length: shader_set.specific_sources.len(),
			shader_uniform_arrays: &shader_uniform_arrays,
			target,
		};
		let shader_declarations =
			self.render_template(Template::ShaderDeclarations, &shader_declarations_context)?;

		#[derive(Serialize)]
		struct ShaderLoadingContext<'a> {
			development: bool,
			shader_codes: &'a ShaderCodes,
			shader_specific_source_0: &'a ShaderSource,
			shader_specific_sources: &'a ShaderSourceMap,
			shader_specific_sources_length: usize,
			shader_uniform_arrays: &'a [ShaderUniformArrayExt<'a>],
			target: BuildTarget,
		}

		let shader_loading_context = ShaderLoadingContext {
			development,
			shader_codes: &shader_codes,
			shader_specific_source_0,
			shader_specific_sources: &shader_set.specific_sources,
			shader_specific_sources_length: shader_set.specific_sources.len(),
			shader_uniform_arrays: &shader_uniform_arrays,
			target,
		};
		let shader_loading =
			self.render_template(Template::ShaderLoading, &shader_loading_context)?;

		#[derive(Serialize)]
		struct OpenGLExtConstant {
			name: String,
			declaration: String,
		}

		#[derive(Serialize)]
		struct OpenGLExtFunction {
			name: String,
			declaration: String,
			typedef_declaration: String,
		}

		let (opengl_ext_constants, opengl_ext_functions) = if !development {
			lazy_static! {
				static ref CONSTANT_RE: Regex = Regex::new(r"\bGL_[A-Z]\w+\b").expect("Bad regex.");
				static ref FUNCTION_RE: Regex = Regex::new(r"\bgl[A-Z]\w+\b").expect("Bad regex.");
			}

			let mut constants = vec![];
			let mut functions = vec![];

			let glew_path = self
				.glew_path
				.as_ref()
				.ok_or("Please set configuration key paths.glew.")?
				.join("include")
				.join("GL")
				.join("glew.h")
				.to_string_lossy()
				.to_string();

			let glew_contents =
				fs::read_to_string(glew_path).map_err(|_| "Failed to read GLEW.".to_string())?;

			{
				let mut parse = |code| {
					for mat in CONSTANT_RE.find_iter(code) {
						let name = mat.as_str();
						if !constants
							.iter()
							.any(|constant: &OpenGLExtConstant| constant.name == name)
						{
							let declaration_re =
								Regex::new(&format!(r"#define {} .+", name)).expect("Bad regex.");
							if let Some(mat) = declaration_re.find(&glew_contents) {
								constants.push(OpenGLExtConstant {
									name: name.to_string(),
									declaration: mat.as_str().to_string(),
								});
							}
						}
					}
					for mat in FUNCTION_RE.find_iter(code) {
						let name = mat.as_str();
						if !functions
							.iter()
							.any(|function: &OpenGLExtFunction| function.name == name)
						{
							let typedef_name = format!("PFN{}PROC", name.to_uppercase());
							let typedef_declaration_re = Regex::new(&format!(
								r"typedef \w+ \(GLAPIENTRY \* {}\).+",
								typedef_name
							))
							.expect("Bad regex.");
							if let Some(mat) = typedef_declaration_re.find(&glew_contents) {
								functions.push(OpenGLExtFunction {
									name: name.to_string(),
									typedef_declaration: mat.as_str().to_string(),
									declaration: format!(
										"#define {} (({})_shibaOpenGLExtFunctions[{}])",
										name,
										typedef_name,
										functions.len()
									),
								});
							}
						}
					}
				};

				parse(&api);
				parse(&render);
				parse(&shader_declarations);
				parse(&shader_loading);

				for code in project_codes.values() {
					parse(code);
				}
			}

			(Some(constants), Some(functions))
		} else {
			(None, None)
		};

		#[derive(Serialize)]
		struct OpenGLDeclarationContext<'a> {
			development: bool,
			opengl_ext_constants: &'a Option<Vec<OpenGLExtConstant>>,
			opengl_ext_functions: &'a Option<Vec<OpenGLExtFunction>>,
		}

		let opengl_declarations_context = OpenGLDeclarationContext {
			development,
			opengl_ext_constants: &opengl_ext_constants,
			opengl_ext_functions: &opengl_ext_functions,
		};
		let opengl_declarations =
			self.render_template(Template::OpenGLDeclarations, &opengl_declarations_context)?;

		#[derive(Serialize)]
		struct OpenGLLoadingContext<'a> {
			development: bool,
			opengl_ext_constants: &'a Option<Vec<OpenGLExtConstant>>,
			opengl_ext_functions: &'a Option<Vec<OpenGLExtFunction>>,
		}

		let opengl_loading_context = OpenGLLoadingContext {
			development,
			opengl_ext_constants: &opengl_ext_constants,
			opengl_ext_functions: &opengl_ext_functions,
		};
		let opengl_loading =
			self.render_template(Template::OpenGLLoading, &opengl_loading_context)?;

		Ok(Contents {
			api,
			opengl_declarations,
			opengl_loading,
			render,
			set_active_uniform_values,
			shader_declarations,
			shader_loading,
		})
	}

	fn render_template<T: Serialize>(
		&self,
		template: Template,
		context: &T,
	) -> Result<String, String> {
		let name = template.name();
		self.tera
			.render(
				&name,
				&Context::from_serialize(&context).map_err(|err| err.to_string())?,
			)
			.map_err(|err| err.to_string())
	}
}

fn to_opengl_type_name(type_name: &str) -> &'static str {
	match type_name {
		"bool" => "GLint",
		"int" => "GLint",
		"float" => "GLfloat",
		"mat2" => "ShibaMat2",
		"mat3" => "ShibaMat3",
		"mat4" => "ShibaMat4",
		"uint" => "GLuint",
		"vec2" => "ShibaVec2",
		"vec3" => "ShibaVec3",
		"vec4" => "ShibaVec4",
		_ => "GLint",
	}
}

fn to_opengl_uniform_call(type_name: &str, first_letter_uppercased_type_name: &str) -> String {
	match type_name {
		"bool" => "glUniform1iv(shibaUniformLocations[SHADER][shibaBoolUniformLocationIndex], shibaBoolUniformCount, shibaBoolUniforms)".to_string(),
		"int" => "glUniform1iv(shibaUniformLocations[SHADER][shibaIntUniformLocationIndex], shibaIntUniformCount, shibaIntUniforms)".to_string(),
		"float" => "glUniform1fv(shibaUniformLocations[SHADER][shibaFloatUniformLocationIndex], shibaFloatUniformCount, shibaFloatUniforms)".to_string(),
		"mat2" => "glUniformMatrix2fv(shibaUniformLocations[SHADER][shibaMat2UniformLocationIndex], shibaMat2UniformCount, GL_FALSE, reinterpret_cast<GLfloat *>(shibaMat2Uniforms))".to_string(),
		"mat3" => "glUniformMatrix3fv(shibaUniformLocations[SHADER][shibaMat3UniformLocationIndex], shibaMat3UniformCount, GL_FALSE, reinterpret_cast<GLfloat *>(shibaMat3Uniforms))".to_string(),
		"mat4" => "glUniformMatrix4fv(shibaUniformLocations[SHADER][shibaMat4UniformLocationIndex], shibaMat4UniformCount, GL_FALSE, reinterpret_cast<GLfloat *>(shibaMat4Uniforms))".to_string(),
		"uint" => "glUniform1iv(shibaUniformLocations[SHADER][shibaUintUniformLocationIndex], shibaUintUniformCount, shibaUintUniforms)".to_string(),
		"vec2" => "glUniform2fv(shibaUniformLocations[SHADER][shibaVec2UniformLocationIndex], shibaVec2UniformCount, reinterpret_cast<GLfloat *>(shibaVec2Uniforms))".to_string(),
		"vec3" => "glUniform3fv(shibaUniformLocations[SHADER][shibaVec3UniformLocationIndex], shibaVec3UniformCount, reinterpret_cast<GLfloat *>(shibaVec3Uniforms))".to_string(),
		"vec4" => "glUniform4fv(shibaUniformLocations[SHADER][shibaVec4UniformLocationIndex], shibaVec4UniformCount, reinterpret_cast<GLfloat *>(shibaVec4Uniforms))".to_string(),
		_ => format!("glUniform1iv(shibaUniformLocations[SHADER][shiba{0}ShaderUniformLocationIndex], shiba{0}ShaderUniformCount, shiba{0}ShaderUniforms)", first_letter_uppercased_type_name),
	}
}

fn to_as_value_name(type_name: &str) -> &'static str {
	match type_name {
		"bool" => "asInt",
		"int" => "asInt",
		"float" => "asFloat",
		"mat2" => "asMat2",
		"mat3" => "asMat3",
		"mat4" => "asMat4",
		"uint" => "asUint",
		"vec2" => "asVec2",
		"vec3" => "asVec3",
		"vec4" => "asVec4",
		_ => "asInt",
	}
}
