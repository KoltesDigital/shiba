mod parsers;
mod types;

use self::types::*;
use super::ShaderMinifier;
use crate::hash_extra;
use crate::parsers::glsl;
use crate::paths::BUILD_ROOT_DIRECTORY;
use crate::types::{Pass, ProjectDescriptor, Sections, ShaderDescriptor, Variable, VariableKind};
use regex::Regex;
use serde::Serialize;
use serde_json;
use std::cell::Cell;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::str;
use tera::Tera;

pub struct ShaderMinifierShaderMinifier<'a> {
	project_descriptor: &'a ProjectDescriptor<'a>,

	exe_path: PathBuf,
	tera: Tera,
}

impl<'a> ShaderMinifierShaderMinifier<'a> {
	pub fn new(project_descriptor: &'a ProjectDescriptor) -> Result<Self, String> {
		let exe_path = project_descriptor
			.configuration
			.paths
			.get("shader-minifier")
			.ok_or("Please set configuration key paths.shader-minifier.")?
			.clone();

		let mut tera = Tera::default();

		tera.add_raw_template("template", include_str!("template.tera"))
			.map_err(|err| err.to_string())?;

		Ok(ShaderMinifierShaderMinifier {
			exe_path,
			project_descriptor,
			tera,
		})
	}
}

const OUTPUT_FILENAME: &str = "shader-descriptor.json";

#[derive(Hash)]
struct Inputs<'a> {
	original_shader_descriptor: &'a ShaderDescriptor,
}

#[derive(Serialize)]
struct Context<'a> {
	pub non_uniform_variables: &'a Vec<&'a mut Variable>,
	pub shader_descriptor: &'a ShaderDescriptor,
}

impl ShaderMinifier for ShaderMinifierShaderMinifier<'_> {
	fn minify(
		&self,
		original_shader_descriptor: &ShaderDescriptor,
	) -> Result<ShaderDescriptor, String> {
		let inputs = Inputs {
			original_shader_descriptor,
		};
		let build_cache_directory = hash_extra::get_build_cache_directory(&inputs)?;
		let build_cache_path = build_cache_directory.join(OUTPUT_FILENAME);

		if !self.project_descriptor.build_options.force && build_cache_path.exists() {
			let json = fs::read_to_string(build_cache_path).map_err(|err| err.to_string())?;
			let shader_descriptor =
				serde_json::from_str(json.as_str()).map_err(|_| "Failed to parse JSON.")?;
			return Ok(shader_descriptor);
		}

		let glsl_version = original_shader_descriptor.glsl_version.clone();
		let mut passes = original_shader_descriptor
			.passes
			.iter()
			.map(|_| Pass::default())
			.collect::<Vec<_>>();
		let mut sections = Sections::default();
		let mut uniform_arrays = original_shader_descriptor.uniform_arrays.clone();
		let mut variables = original_shader_descriptor.variables.clone();

		let mut non_uniform_variables = variables
			.iter_mut()
			.filter(|variable| {
				variable.active
					&& match variable.kind {
						VariableKind::Uniform(_) => false,
						_ => true,
					}
			})
			.collect::<Vec<_>>();

		let context = Context {
			non_uniform_variables: &non_uniform_variables,
			shader_descriptor: &original_shader_descriptor,
		};

		let shader = self
			.tera
			.render("template", &context)
			.map_err(|_| "Failed to render template.")?;

		let build_directory = BUILD_ROOT_DIRECTORY
			.join("shader-minifiers")
			.join("shader-minifier");
		fs::create_dir_all(&build_directory).map_err(|err| err.to_string())?;

		let input_path = build_directory.join("shader.glsl");
		let output_path = build_directory.join("shader.min.glsl");

		fs::write(&input_path, shader).map_err(|_| "Failed to write shader.")?;

		let mut minification = Command::new(&self.exe_path)
			.args(vec!["--field-names", "rgba", "--format", "none", "-o"])
			.arg(&output_path)
			.args(vec!["-v", "--"])
			.arg(&input_path)
			.current_dir(&build_directory)
			.stdout(Stdio::inherit())
			.stderr(Stdio::inherit())
			.spawn()
			.map_err(|err| err.to_string())?;

		let status = minification.wait().map_err(|err| err.to_string())?;
		if !status.success() {
			return Err("Failed to compile.".to_string());
		}

		let contents = fs::read_to_string(&output_path)
			.map_err(|_| "Failed to read shader.".to_string())?
			.replace("\r", "");

		let (input, contents) =
			parsers::contents(&contents).map_err(|_| "Parsing error.".to_string())?;

		let mut uniform_arrays_string = None;
		let mut non_uniform_variables_string = None;
		let next_section = Cell::from(None);

		let mut process_code = |code: &str| {
			if let Some(next_section) = next_section.get() {
				let code = code.trim();
				if !code.is_empty() {
					let mut code = code.to_string();

					// HACK https://github.com/laurentlb/Shader_Minifier/issues/19
					for uniform_array in &uniform_arrays {
						if let Some(minified_name) = &uniform_array.minified_name {
							let usage_re = Regex::new(&format!(r"\b{}\b", uniform_array.name))
								.expect("Bad regex.");
							code = usage_re
								.replace_all(&code, minified_name.as_str())
								.to_string();
						}
					}

					let code = Some(code);

					match next_section {
						Directive::Attributes => sections.attributes = code,

						Directive::Common => sections.common = code,

						Directive::Fragment(index) => {
							passes[index].fragment = code;
						}

						Directive::Outputs => sections.outputs = code,

						Directive::UniformArrays => uniform_arrays_string = code,

						Directive::Variables => non_uniform_variables_string = code,

						Directive::Varyings => sections.varyings = code,

						Directive::Vertex(index) => {
							passes[index].vertex = code;
						}
					}
				}
			}
		};

		for (code, section) in contents {
			process_code(code);
			next_section.set(Some(section));
		}

		process_code(input);

		for (index, variable) in glsl::variables(&uniform_arrays_string.unwrap())
			.unwrap()
			.1
			.iter()
			.enumerate()
		{
			uniform_arrays[index].minified_name = Some(variable.name.clone());
		}

		if !non_uniform_variables.is_empty() {
			for (index, variable) in glsl::variables(&non_uniform_variables_string.unwrap())
				.unwrap()
				.1
				.iter()
				.enumerate()
			{
				non_uniform_variables[index].minified_name = Some(variable.name.clone());
			}
		}

		let shader_descriptor = ShaderDescriptor {
			glsl_version,
			passes,
			sections,
			uniform_arrays,
			variables,
		};

		let json = serde_json::to_string(&shader_descriptor).map_err(|_| "Failed to dump JSON.")?;
		fs::write(build_cache_path, json).map_err(|err| err.to_string())?;

		Ok(shader_descriptor)
	}
}
