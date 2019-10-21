use crate::config_provider::ConfigProvider;
use crate::directories;
use crate::template::{Template, TemplateRenderer};
use crate::types::Pass;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;
use std::str;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConfigLink {
	pub args: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConfigPaths {
	pub glew: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Config {
	pub link: ConfigLink,
	pub paths: ConfigPaths,
}

#[derive(Debug, Serialize)]
struct APIContext {
	pub blender_api: bool,
	pub development: bool,
}

#[derive(Debug, Serialize)]
struct BlenderAPIContext<'a> {
	pub api: &'a String,
	pub passes: &'a [Pass],
	pub render: &'a String,
	pub shader_declarations: &'a String,
	pub shader_loading: &'a String,
}

#[derive(Debug, Serialize)]
struct ShaderDeclarationContext<'a> {
	pub passes: &'a [Pass],
}

#[derive(Debug, Serialize)]
struct ShaderLoadingContext<'a> {
	pub passes: &'a [Pass],
}

#[derive(Debug)]
pub struct BlenderAPIGenerator {
	config: Config,
}

impl BlenderAPIGenerator {
	pub fn new(config_provider: &ConfigProvider) -> Result<Self, String> {
		let config = config_provider.try_into()?;
		Ok(BlenderAPIGenerator { config })
	}

	pub fn generate(
		&self,
		template_renderer: &TemplateRenderer,
		passes: &[Pass],
	) -> Result<(), String> {
		let api_context = APIContext {
			blender_api: true,
			development: true,
		};
		let api_contents = template_renderer.render_context(Template::API, &api_context)?;

		let render_contents = template_renderer.render(Template::Render)?;

		let shader_declarations_context = ShaderDeclarationContext { passes };
		let shader_declarations_contents = template_renderer
			.render_context(Template::ShaderDeclarations, &shader_declarations_context)?;

		let shader_loading_context = ShaderLoadingContext { passes };
		let shader_loading_contents =
			template_renderer.render_context(Template::ShaderLoading, &shader_loading_context)?;

		let blender_empty_context = BlenderAPIContext {
			api: &api_contents,
			passes,
			render: &render_contents,
			shader_declarations: &shader_declarations_contents,
			shader_loading: &shader_loading_contents,
		};
		let blender_api_contents =
			template_renderer.render_context(Template::BlenderAPI, &blender_empty_context)?;

		{
			let mut file = File::create((*directories::TMP).join("blender_api.cpp"))
				.map_err(|_| "Failed to create blender_api.cpp.")?;
			file.write_all(blender_api_contents.as_bytes())
				.map_err(|_| "Failed to write to file.")?;
			file.sync_data().map_err(|_| "Failed to sync file.")?;
		}

		let obj = "blender_api.obj";
		let cl = Command::new("cl.exe")
			.current_dir(&*directories::TMP)
			.arg(format!(
				"/I{}",
				PathBuf::from(&self.config.paths.glew)
					.join("include")
					.to_string_lossy()
			))
			.arg("/FA")
			.arg(format!("/Fa{}.asm", obj))
			.arg("/c")
			.arg(format!("/Fo{}", obj))
			.args(&["blender_api.cpp"])
			.output()
			.map_err(|_| "Failed to execute cl.")?;
		println!(
			"{}",
			str::from_utf8(&cl.stdout).map_err(|_| "Failed to convert UTF8.")?
		);

		if !cl.status.success() {
			return Err("Failed to compile".to_string());
		}

		let link = Command::new("link.exe")
			.current_dir(&*directories::TMP)
			.arg("/DLL")
			.arg("/OUT:blender_api.dll")
			.args(&self.config.link.args)
			.arg(format!(
				"{}",
				PathBuf::from(&self.config.paths.glew)
					.join("lib")
					.join("Release")
					.join("x64")
					.join("glew32s.lib")
					.to_string_lossy()
			))
			.arg("blender_api.obj")
			.output()
			.map_err(|_| "Failed to execute link.")?;
		println!(
			"{}",
			str::from_utf8(&link.stdout).map_err(|_| "Failed to convert UTF8.")?
		);

		Ok(())
	}
}
