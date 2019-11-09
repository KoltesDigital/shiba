mod settings;

pub use self::settings::Settings;
use crate::configuration::Configuration;
use crate::paths::{self, TEMP_DIRECTORY};
use crate::shader_codes::ShaderCodes;
use crate::template::{Template, TemplateRenderer};
use crate::types::{Pass, ShaderDescriptor, UniformArray};
use serde::Serialize;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

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
	pub uniform_arrays: &'a [UniformArrayElement<'a>],
}

#[derive(Debug, Serialize)]
struct RenderContext<'a> {
	pub blender_api: bool,
	pub custom: &'a HashMap<String, String>,
}

#[derive(Debug, Serialize)]
struct ShaderDeclarationContext<'a> {
	pub shader_codes: &'a ShaderCodes,
	pub passes: &'a [Pass],
	pub uniform_arrays: &'a [UniformArrayElement<'a>],
}

#[derive(Debug, Serialize)]
struct ShaderLoadingContext<'a> {
	pub blender_api: bool,
	pub shader_codes: &'a ShaderCodes,
	pub development: bool,
	pub passes: &'a [Pass],
	pub uniform_arrays: &'a [UniformArrayElement<'a>],
}

#[derive(Debug)]
pub struct BlenderAPIGenerator<'a> {
	settings: &'a Settings,
	glew_path: PathBuf,
	msvc_path: PathBuf,
}

impl<'a> BlenderAPIGenerator<'a> {
	pub fn new(settings: &'a Settings, configuration: &'a Configuration) -> Result<Self, String> {
		let glew_path = configuration
			.paths
			.glew
			.clone()
			.ok_or("Please set configuration key paths.glew.")?;
		let msvc_path = paths::msvc(paths::MSVCPlatform::X64)?;
		Ok(BlenderAPIGenerator {
			settings,
			glew_path,
			msvc_path,
		})
	}

	pub fn generate(
		&self,
		project_directory: &Path,
		template_renderer: &TemplateRenderer,
		shader_descriptor: &ShaderDescriptor,
	) -> Result<(), String> {
		let passes = &shader_descriptor.passes;

		let uniform_arrays = shader_descriptor
			.uniform_arrays
			.iter()
			.map(|e| UniformArrayElement {
				type_name: e.0,
				uniform_array: e.1,
			})
			.collect::<Vec<UniformArrayElement>>();

		let custom = fs::read_dir(&project_directory)
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

		let shader_codes = ShaderCodes::load(shader_descriptor);

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
			shader_codes: &shader_codes,
			passes,
			uniform_arrays: &uniform_arrays,
		};
		let shader_declarations_contents = template_renderer
			.render_context(Template::ShaderDeclarations, &shader_declarations_context)?;

		let shader_loading_context = ShaderLoadingContext {
			blender_api: true,
			shader_codes: &shader_codes,
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
			uniform_arrays: &uniform_arrays,
		};
		let blender_api_contents =
			template_renderer.render_context(Template::BlenderAPI, &blender_empty_context)?;

		fs::write(
			TEMP_DIRECTORY.join("blender_api.cpp"),
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
				PathBuf::from(&self.glew_path)
					.join("include")
					.to_string_lossy(),
			))
			.arg("blender_api.cpp")
			.arg("&&")
			.arg("link")
			.arg("/DLL")
			.arg("/OUT:blender_api.dll")
			.args(&self.settings.link.args)
			.arg(
				PathBuf::from(&self.glew_path)
					.join("lib")
					.join("Release")
					.join("x64")
					.join("glew32s.lib")
					.to_string_lossy()
					.to_string(),
			)
			.arg("blender_api.obj")
			.current_dir(&*paths::TEMP_DIRECTORY)
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

		Ok(())
	}

	pub fn get_path() -> PathBuf {
		TEMP_DIRECTORY.join("blender_api.dll")
	}
}
