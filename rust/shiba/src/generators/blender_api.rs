use crate::config_provider::ConfigProvider;
use crate::paths;
use crate::template::{Template, TemplateRenderer};
use crate::types::{Pass, ProjectDescriptor, ShaderDescriptor, UniformArray, VariableKind};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::str;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct LinkConfig {
	pub args: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct PathsConfig {
	pub glew: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Config {
	pub link: LinkConfig,
	pub paths: PathsConfig,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
struct DefaultLinkConfig {
	pub args: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
struct DefaultPathsConfig {}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
struct DefaultConfig {
	pub link: DefaultLinkConfig,
	pub paths: DefaultPathsConfig,
}

#[derive(Debug, Default, Serialize)]
struct CommonCodes {
	pub after_stage_variables: String,
	pub before_stage_variables: String,
	pub fragment_specific: String,
	pub vertex_specific: String,
}

#[derive(Debug, Serialize)]
struct UniformArrayElement<'a> {
	pub type_name: &'a String,
	pub uniform_array: &'a UniformArray,
}

#[derive(Debug, Serialize)]
struct APIContext<'a> {
	pub blender_api: bool,
	pub development: bool,
	pub passes: &'a [Pass],
}

#[derive(Debug, Serialize)]
struct BlenderAPIContext<'a> {
	pub api: &'a String,
	pub custom: &'a HashMap<String, String>,
	pub passes: &'a [Pass],
	pub render: &'a String,
	pub shader_declarations: &'a String,
	pub shader_loading: &'a String,
}

#[derive(Debug, Serialize)]
struct RenderContext<'a> {
	pub blender_api: bool,
	pub custom: &'a HashMap<String, String>,
}

#[derive(Debug, Serialize)]
struct ShaderDeclarationContext<'a> {
	pub common_codes: &'a CommonCodes,
	pub passes: &'a [Pass],
	pub uniform_arrays: &'a [UniformArrayElement<'a>],
}

#[derive(Debug, Serialize)]
struct ShaderLoadingContext<'a> {
	pub blender_api: bool,
	pub common_codes: &'a CommonCodes,
	pub development: bool,
	pub passes: &'a [Pass],
	pub uniform_arrays: &'a [UniformArrayElement<'a>],
}

#[derive(Debug)]
pub struct BlenderAPIGenerator {
	config: Config,
	msvc_path: PathBuf,
}

impl BlenderAPIGenerator {
	pub fn new(config_provider: &ConfigProvider) -> Result<Self, String> {
		let config = config_provider.get_default(DefaultConfig {
			link: DefaultLinkConfig {
				args: vec!["/MACHINE:X64", "gdi32.lib", "opengl32.lib", "user32.lib"],
			},
			paths: DefaultPathsConfig {},
		})?;
		let msvc_path = paths::msvc(paths::MSVCPlatform::X64)?;
		Ok(BlenderAPIGenerator { config, msvc_path })
	}

	pub fn generate(
		&self,
		template_renderer: &TemplateRenderer,
		project_descriptor: &ProjectDescriptor,
		shader_descriptor: &ShaderDescriptor,
	) -> Result<String, String> {
		let passes = &shader_descriptor.passes;

		let uniform_arrays = shader_descriptor
			.uniform_arrays
			.iter()
			.map(|e| UniformArrayElement {
				type_name: e.0,
				uniform_array: e.1,
			})
			.collect::<Vec<UniformArrayElement>>();

		let custom = fs::read_dir(&project_descriptor.directory)
			.map_err(|_| "Failed to read directory.")?
			.filter_map(|entry| {
				let entry = entry.unwrap();
				if entry.file_type().unwrap().is_file() {
					let path = entry.path();
					if path.extension() == Some(OsStr::new("cpp")) {
						let name = path
							.file_stem()
							.unwrap()
							.to_str()
							.expect("Failed to convert path.")
							.to_string();
						let contents = fs::read_to_string(&path).expect("Failed to read file.");
						Some((name, contents))
					} else {
						None
					}
				} else {
					None
				}
			})
			.collect::<HashMap<String, String>>();

		let mut common_codes = CommonCodes::default();
		let mut vertex_location_index = 0;

		lazy_static! {
			static ref STAGE_VARIABLE_RE: Regex = Regex::new(r"\w+ [\w,]+;").expect("Bad regex.");
		}

		if let Some(code) = &shader_descriptor.sections.attributes {
			for mat in STAGE_VARIABLE_RE.find_iter(code.as_str()) {
				common_codes.vertex_specific += format!(
					"layout(location={})in {}",
					vertex_location_index,
					mat.as_str()
				)
				.as_str();
				vertex_location_index += 1;
			}
		}

		if let Some(code) = &shader_descriptor.sections.varyings {
			for mat in STAGE_VARIABLE_RE.find_iter(code.as_str()) {
				common_codes.vertex_specific += format!("out {}", mat.as_str()).as_str();
				common_codes.fragment_specific += format!("in {}", mat.as_str()).as_str();
			}
		}

		if let Some(code) = &shader_descriptor.sections.outputs {
			for mat in STAGE_VARIABLE_RE.find_iter(code.as_str()) {
				common_codes.fragment_specific += format!("out {}", mat.as_str()).as_str();
			}
		}

		if let Some(version) = &shader_descriptor.glsl_version {
			common_codes.before_stage_variables = format!("#version {}\n", version);
		}

		let mut globals_by_type = HashMap::new();
		for variable in &shader_descriptor.variables {
			if !variable.active {
				continue;
			}

			match variable.kind {
				VariableKind::Uniform => {}
				_ => {
					if !globals_by_type.contains_key(&variable.type_name) {
						let _ = globals_by_type.insert(variable.type_name.clone(), Vec::new());
					}

					let mut name = variable
						.minified_name
						.as_ref()
						.unwrap_or(&variable.name)
						.clone();
					if let VariableKind::Const(value) = &variable.kind {
						name += format!(" = {}", value).as_str();
					}
					globals_by_type
						.get_mut(&variable.type_name)
						.unwrap()
						.push(name);
				}
			}
		}

		for pair in &shader_descriptor.uniform_arrays {
			common_codes.after_stage_variables += format!(
				"uniform {} {}[{}];",
				pair.0,
				pair.1.minified_name.as_ref().unwrap_or(&pair.1.name),
				pair.1.variables.len()
			)
			.as_str();
		}

		for pair in &globals_by_type {
			common_codes.after_stage_variables +=
				format!("{} {};", pair.0, pair.1.join("-")).as_str();
		}

		if let Some(code) = &shader_descriptor.sections.common {
			common_codes.after_stage_variables += code.as_str();
		}

		if !common_codes.before_stage_variables.is_empty()
			&& common_codes.vertex_specific.is_empty()
			&& common_codes.fragment_specific.is_empty()
		{
			common_codes.after_stage_variables =
				common_codes.before_stage_variables + common_codes.after_stage_variables.as_str();
			common_codes.before_stage_variables = String::new();
		}

		let api_context = APIContext {
			blender_api: true,
			development: true,
			passes,
		};
		let api_contents = template_renderer.render_context(Template::API, &api_context)?;

		let render_context = RenderContext {
			blender_api: true,
			custom: &custom,
		};
		let render_contents =
			template_renderer.render_context(Template::Render, &render_context)?;

		let shader_declarations_context = ShaderDeclarationContext {
			common_codes: &common_codes,
			passes,
			uniform_arrays: &uniform_arrays,
		};
		let shader_declarations_contents = template_renderer
			.render_context(Template::ShaderDeclarations, &shader_declarations_context)?;

		let shader_loading_context = ShaderLoadingContext {
			blender_api: true,
			common_codes: &common_codes,
			development: true,
			passes,
			uniform_arrays: &uniform_arrays,
		};
		let shader_loading_contents =
			template_renderer.render_context(Template::ShaderLoading, &shader_loading_context)?;

		let blender_empty_context = BlenderAPIContext {
			api: &api_contents,
			custom: &custom,
			passes,
			render: &render_contents,
			shader_declarations: &shader_declarations_contents,
			shader_loading: &shader_loading_contents,
		};
		let blender_api_contents =
			template_renderer.render_context(Template::BlenderAPI, &blender_empty_context)?;

		fs::write(
			(*paths::TEMP_DIR).join("blender_api.cpp"),
			blender_api_contents.as_bytes(),
		)
		.map_err(|_| "Failed to write to file.")?;

		let compilation = Command::new("cmd.exe")
			.arg("/c")
			.arg("call")
			.arg(format!(
				r#"{}\VC\Auxiliary\Build\vcvars64.bat"#,
				self.msvc_path.to_string_lossy(),
			))
			.arg("&&")
			.arg("cl")
			.arg("/c")
			.arg("/EHsc")
			.arg("/FA")
			.arg("/Fablender_api.asm")
			.arg("/Foblender_api.obj")
			.arg(format!(
				"/I{}",
				PathBuf::from(&self.config.paths.glew)
					.join("include")
					.to_string_lossy(),
			))
			.arg("blender_api.cpp")
			.arg("&&")
			.arg("link")
			.arg("/DLL")
			.arg("/OUT:blender_api.dll")
			.args(&self.config.link.args)
			.arg(
				PathBuf::from(&self.config.paths.glew)
					.join("lib")
					.join("Release")
					.join("x64")
					.join("glew32s.lib")
					.to_string_lossy()
					.to_string(),
			)
			.arg("blender_api.obj")
			.current_dir(&*paths::TEMP_DIR)
			.output()
			.map_err(|err| err.to_string())?;

		println!(
			"stdout: {}",
			str::from_utf8(&compilation.stdout).map_err(|_| "Failed to convert UTF8.")?
		);

		println!(
			"stderr: {}",
			str::from_utf8(&compilation.stderr).map_err(|_| "Failed to convert UTF8.")?
		);

		if !compilation.status.success() {
			return Err("Failed to compile".to_string());
		}

		Ok((*paths::TEMP_DIR)
			.join("blender_api.dll")
			.to_string_lossy()
			.to_string())
	}
}
