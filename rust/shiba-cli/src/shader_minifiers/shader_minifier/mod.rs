mod parsers;
mod types;

use self::types::*;
use super::ShaderMinifier;
use crate::build::BuildOptions;
use crate::hash_extra;
use crate::parsers::glsl;
use crate::paths::BUILD_ROOT_DIRECTORY;
use crate::project_data::Project;
use crate::shader_data::{
	ShaderProgram, ShaderProgramMap, ShaderSections, ShaderSet, ShaderVariable, ShaderVariableKind,
};
use regex::Regex;
use serde::Serialize;
use serde_json;
use std::cell::Cell;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::str;
use tera::{Context, Tera};

pub struct ShaderMinifierShaderMinifier {
	exe_path: PathBuf,
	tera: Tera,
}

impl ShaderMinifierShaderMinifier {
	pub fn new(project: &Project) -> Result<Self, String> {
		let exe_path = project
			.configuration
			.paths
			.get("shader-minifier")
			.ok_or("Please set configuration key paths.shader-minifier.")?
			.clone();

		let mut tera = Tera::default();

		tera.add_raw_template(
			"shader-minifier-shader-minifier",
			include_str!("template.tera"),
		)
		.map_err(|err| err.to_string())?;

		Ok(ShaderMinifierShaderMinifier { exe_path, tera })
	}
}

impl ShaderMinifier for ShaderMinifierShaderMinifier {
	fn minify(
		&self,
		build_options: &BuildOptions,
		original_shader_set: &ShaderSet,
	) -> Result<ShaderSet, String> {
		const OUTPUT_FILENAME: &str = "shader-descriptor.json";

		#[derive(Hash)]
		struct Inputs<'a> {
			exe_path: &'a Path,
			original_shader_set: &'a ShaderSet,
		}

		let inputs = Inputs {
			exe_path: &self.exe_path,
			original_shader_set,
		};
		let build_cache_directory = hash_extra::get_build_cache_directory(&inputs)?;
		let build_cache_path = build_cache_directory.join(OUTPUT_FILENAME);

		if !build_options.force && build_cache_path.exists() {
			let json = fs::read_to_string(build_cache_path).map_err(|err| err.to_string())?;
			let shader_set =
				serde_json::from_str(json.as_str()).map_err(|_| "Failed to parse JSON.")?;
			return Ok(shader_set);
		}

		let glsl_version = original_shader_set.glsl_version.clone();
		let mut programs = original_shader_set
			.programs
			.iter()
			.map(|(name, _)| (name.clone(), ShaderProgram::default()))
			.collect::<ShaderProgramMap>();
		let mut sections = ShaderSections::default();
		let mut uniform_arrays = original_shader_set.uniform_arrays.clone();
		let mut variables = original_shader_set.variables.clone();

		let mut non_uniform_variables = variables
			.iter_mut()
			.filter(|variable| {
				variable.active
					&& match variable.kind {
						ShaderVariableKind::Uniform(_) => false,
						_ => true,
					}
			})
			.collect::<Vec<_>>();

		#[derive(Serialize)]
		struct OwnContext<'a> {
			pub non_uniform_variables: &'a Vec<&'a mut ShaderVariable>,
			pub shader_set: &'a ShaderSet,
		}

		let context = OwnContext {
			non_uniform_variables: &non_uniform_variables,
			shader_set: &original_shader_set,
		};

		let shader = self
			.tera
			.render(
				"shader-minifier-shader-minifier",
				&Context::from_serialize(&context).map_err(|err| err.to_string())?,
			)
			.map_err(|err| err.to_string())?;

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

						Directive::Fragment(name) => {
							programs.get_mut(name).unwrap().fragment = code;
						}

						Directive::Outputs => sections.outputs = code,

						Directive::ShaderUniformArrays => uniform_arrays_string = code,

						Directive::ShaderVariables => non_uniform_variables_string = code,

						Directive::Varyings => sections.varyings = code,

						Directive::Vertex(name) => {
							programs.get_mut(name).unwrap().vertex = code;
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

		let shader_set = ShaderSet {
			glsl_version,
			programs,
			sections,
			uniform_arrays,
			variables,
		};

		let json = serde_json::to_string(&shader_set).map_err(|_| "Failed to dump JSON.")?;
		fs::write(build_cache_path, json).map_err(|err| err.to_string())?;

		Ok(shader_set)
	}
}
