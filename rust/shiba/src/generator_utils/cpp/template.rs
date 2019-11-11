use crate::configuration::Configuration;
use crate::shader_codes::ShaderCodes;
use crate::types::{Pass, ShaderDescriptor, UniformArray};
use regex::Regex;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use tera::{Tera, Value};

macro_rules! template_enum {
	(
		$($variant:ident: $filename:expr),*,
	) => {
		enum Template {
			$($variant),*
		}

		impl Template {
			fn as_array() -> Vec<(&'static str, &'static str)> {
				vec![
					$((stringify!($variant), include_str!(concat!("templates/", $filename, ".tera")))),*
				]
			}

			fn name(&self) -> &'static str {
				match self {
					$(Template::$variant => stringify!($variant)),*
				}
			}
		}
	};
}

template_enum! {
	API: "api",
	OpenGLDeclarations: "opengl_declarations",
	OpenGLLoading: "opengl_loading",
	Render: "render",
	ShaderDeclarations: "shader_declarations",
	ShaderLoading: "shader_loading",
}

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

#[derive(Serialize)]
struct APIContext<'a> {
	development: bool,
	passes: &'a [Pass],
	target: &'a str,
}

#[derive(Serialize)]
struct OpenGLDeclarationContext<'a> {
	development: bool,
	opengl_ext_constants: &'a Option<Vec<OpenGLExtConstant>>,
	opengl_ext_functions: &'a Option<Vec<OpenGLExtFunction>>,
}

#[derive(Serialize)]
struct OpenGLLoadingContext<'a> {
	development: bool,
	opengl_ext_constants: &'a Option<Vec<OpenGLExtConstant>>,
	opengl_ext_functions: &'a Option<Vec<OpenGLExtFunction>>,
}

#[derive(Serialize)]
struct RenderContext<'a> {
	custom_codes: &'a BTreeMap<String, String>,
	target: &'a str,
}

#[derive(Serialize)]
struct ShaderDeclarationContext<'a> {
	passes: &'a [Pass],
	shader_codes: &'a ShaderCodes,
	uniform_arrays: &'a [UniformArray],
}

#[derive(Serialize)]
struct ShaderLoadingContext<'a> {
	development: bool,
	passes: &'a [Pass],
	shader_codes: &'a ShaderCodes,
	target: &'a str,
	uniform_arrays: &'a [UniformArray],
}

pub struct Contents {
	pub api: String,
	pub opengl_declarations: String,
	pub opengl_loading: String,
	pub render: String,
	pub shader_declarations: String,
	pub shader_loading: String,
}

pub struct TemplateRenderer {
	glew_path: Option<PathBuf>,
	tera: Tera,
}

impl TemplateRenderer {
	pub fn new(configuration: &Configuration) -> Result<Self, String> {
		let glew_path = configuration.paths.get("glew").cloned();

		let mut tera = Tera::default();

		tera.add_raw_templates(Template::as_array())
			.map_err(|err| err.to_string())?;

		tera.register_filter("string_literal", |value, args| match value {
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
		});

		tera.register_filter("to_opengl_type", |value, _| match value {
			Value::String(old) => {
				let new = match old.as_str() {
					"bool" => "GLint",
					"int" => "GLint",
					"uint" => "GLuint",
					"float" => "GLfloat",
					_ => "GLint",
				};
				Ok(Value::String(new.to_string()))
			}
			_ => Err(tera::Error::from("to_opengl_type expects a string")),
		});

		tera.register_filter("uppercase_first", |value, _| match value {
			Value::String(old) => {
				let mut new = old;
				if let Some(c) = new.get_mut(0..1) {
					c.make_ascii_uppercase();
				}
				Ok(Value::String(new))
			}
			_ => Err(tera::Error::from("uppercase_first expects a string")),
		});

		Ok(TemplateRenderer { glew_path, tera })
	}

	pub fn render(
		&self,
		custom_codes: &BTreeMap<String, String>,
		shader_descriptor: &ShaderDescriptor,
		development: bool,
		target: &str,
	) -> Result<Contents, String> {
		let shader_codes = ShaderCodes::load(shader_descriptor);

		let api_context = APIContext {
			development,
			passes: &shader_descriptor.passes,
			target,
		};
		let api = self.render_template(Template::API, &api_context)?;

		let render_context = RenderContext {
			custom_codes: &custom_codes,
			target,
		};
		let render = self.render_template(Template::Render, &render_context)?;

		let shader_declarations_context = ShaderDeclarationContext {
			shader_codes: &shader_codes,
			passes: &shader_descriptor.passes,
			uniform_arrays: &shader_descriptor.uniform_arrays,
		};
		let shader_declarations =
			self.render_template(Template::ShaderDeclarations, &shader_declarations_context)?;

		let shader_loading_context = ShaderLoadingContext {
			development,
			passes: &shader_descriptor.passes,
			shader_codes: &shader_codes,
			target,
			uniform_arrays: &shader_descriptor.uniform_arrays,
		};
		let shader_loading =
			self.render_template(Template::ShaderLoading, &shader_loading_context)?;

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

				for code in custom_codes.values() {
					parse(code);
				}
			}

			(Some(constants), Some(functions))
		} else {
			(None, None)
		};

		let opengl_declarations_context = OpenGLDeclarationContext {
			development,
			opengl_ext_constants: &opengl_ext_constants,
			opengl_ext_functions: &opengl_ext_functions,
		};
		let opengl_declarations =
			self.render_template(Template::OpenGLDeclarations, &opengl_declarations_context)?;

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
			.render(&name, context)
			.map_err(|_| format!("Failed to render {}.", name))
	}
}
