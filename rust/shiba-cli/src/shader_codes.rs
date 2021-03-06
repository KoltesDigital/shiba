use crate::shader_data::{ShaderConstVariable, ShaderSet, ShaderVariableKind};
use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Default, Serialize)]
pub struct ShaderCodes {
	pub before_stage_variables: String,
	pub after_stage_variables: String,

	pub fragment_specific: String,
	pub vertex_specific: String,
}

impl ShaderCodes {
	pub fn load(shader_set: &ShaderSet) -> ShaderCodes {
		let mut shader_codes = ShaderCodes::default();
		let mut vertex_location_index = 0;

		lazy_static! {
			static ref STAGE_VARIABLE_RE: Regex = Regex::new(r"\w+ [\w,]+;").expect("Bad regex.");
		}

		if let Some(code) = &shader_set.sections.attributes {
			for mat in STAGE_VARIABLE_RE.find_iter(code.as_str()) {
				shader_codes.vertex_specific += format!(
					"layout(location={})in {}",
					vertex_location_index,
					mat.as_str()
				)
				.as_str();
				vertex_location_index += 1;
			}
		}

		if let Some(code) = &shader_set.sections.varyings {
			for mat in STAGE_VARIABLE_RE.find_iter(code.as_str()) {
				shader_codes.vertex_specific += format!("out {}", mat.as_str()).as_str();
				shader_codes.fragment_specific += format!("in {}", mat.as_str()).as_str();
			}
		}

		if let Some(code) = &shader_set.sections.outputs {
			for mat in STAGE_VARIABLE_RE.find_iter(code.as_str()) {
				shader_codes.fragment_specific += format!("out {}", mat.as_str()).as_str();
			}
		}

		if let Some(version) = &shader_set.glsl_version {
			shader_codes.before_stage_variables = format!("#version {}\n", version);
		}

		let mut globals_by_type = HashMap::new();
		for variable in &shader_set.variables {
			if !variable.active {
				continue;
			}

			match variable.kind {
				ShaderVariableKind::Uniform(_) => {}
				_ => {
					if !globals_by_type.contains_key(&variable.type_name) {
						let _ = globals_by_type.insert(variable.type_name.clone(), Vec::new());
					}

					let mut name = variable
						.minified_name
						.as_ref()
						.unwrap_or(&variable.name)
						.clone();
					if let ShaderVariableKind::Const(ShaderConstVariable { value }) = &variable.kind
					{
						name += format!(" = {}", value).as_str();
					}
					globals_by_type
						.get_mut(&variable.type_name)
						.unwrap()
						.push(name);
				}
			}
		}

		for uniform_array in &shader_set.uniform_arrays {
			shader_codes.after_stage_variables += format!(
				"uniform {} {}[{}];",
				uniform_array.type_name,
				uniform_array
					.minified_name
					.as_ref()
					.unwrap_or(&uniform_array.name),
				uniform_array.variables.len()
			)
			.as_str();
		}

		for (type_name, variables) in &globals_by_type {
			shader_codes.after_stage_variables +=
				format!("{} {};", type_name, variables.join(",")).as_str();
		}

		if let Some(code) = &shader_set.sections.common {
			shader_codes.after_stage_variables += code.as_str();
		}

		if !shader_codes.before_stage_variables.is_empty()
			&& shader_codes.vertex_specific.is_empty()
			&& shader_codes.fragment_specific.is_empty()
		{
			shader_codes.after_stage_variables =
				shader_codes.before_stage_variables + shader_codes.after_stage_variables.as_str();
			shader_codes.before_stage_variables = String::new();
		}

		shader_codes
	}
}
