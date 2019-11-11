mod parsers;
mod types;

use self::types::*;
use crate::configuration::Configuration;
use crate::parsers::glsl;
use crate::paths::TEMP_DIRECTORY;
use crate::traits;
use crate::types::{Pass, Sections, ShaderDescriptor, Variable, VariableKind};
use regex::Regex;
use serde::Serialize;
use std::cell::Cell;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::str;
use tera::Tera;

pub struct ShaderMinifier {
	exe_path: PathBuf,
	tera: Tera,
}

impl ShaderMinifier {
	pub fn new(configuration: &Configuration) -> Result<Self, String> {
		let exe_path = configuration
			.paths
			.get("shader-minifier")
			.ok_or("Please set configuration key paths.shader-minifier.")?
			.clone();

		let mut tera = Tera::default();

		tera.add_raw_template("template", include_str!("template.tera"))
			.map_err(|err| err.to_string())?;

		Ok(ShaderMinifier { exe_path, tera })
	}
}

#[derive(Serialize)]
struct Context<'a> {
	pub non_uniform_variables: &'a Vec<&'a mut Variable>,
	pub shader_descriptor: &'a ShaderDescriptor,
}

impl traits::ShaderMinifier for ShaderMinifier {
	fn minify(
		&self,
		original_shader_descriptor: &ShaderDescriptor,
	) -> Result<ShaderDescriptor, String> {
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
						VariableKind::Uniform => false,
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

		let input_path = TEMP_DIRECTORY.join("shader.glsl");
		let output_path = TEMP_DIRECTORY.join("shader.min.glsl");

		fs::write(&input_path, shader).map_err(|_| "Failed to write shader.")?;

		let mut minification = Command::new(&self.exe_path)
			.args(vec!["--field-names", "rgba", "--format", "none", "-o"])
			.arg(&output_path)
			.args(vec!["-v", "--"])
			.arg(&input_path)
			.current_dir(&*TEMP_DIRECTORY)
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
						Section::Attributes => sections.attributes = code,

						Section::Common => sections.common = code,

						Section::Fragment(index) => {
							passes[index].fragment = code;
						}

						Section::Outputs => sections.outputs = code,

						Section::UniformArrays => uniform_arrays_string = code,

						Section::Variables => non_uniform_variables_string = code,

						Section::Varyings => sections.varyings = code,

						Section::Vertex(index) => {
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
		Ok(shader_descriptor)
	}
}
