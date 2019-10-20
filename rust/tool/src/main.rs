#[macro_use]
extern crate lazy_static;

mod config;
mod directories;
mod templates;

use config::Config;
use serde::Serialize;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;
use std::str;
use tera;
use tera::{Context, Tera, Value};

static FRAGMENT: &str = include_str!("shader.frag");

#[derive(Debug, Default, Serialize)]
struct Pass {
	pub fragment: Option<String>,
	pub vertex: Option<String>,
}

#[derive(Debug, Serialize)]
struct APIContext {
	pub blender_api: bool,
	pub development: bool,
}

#[derive(Debug, Serialize)]
struct BlenderAPIContext<'a> {
	pub api: &'a String,
	pub passes: &'a Vec<Pass>,
	pub render: &'a String,
	pub shader_declarations: &'a String,
	pub shader_loading: &'a String,
}

#[derive(Debug, Serialize)]
struct ShaderDeclarationContext<'a> {
	pub passes: &'a Vec<Pass>,
}

#[derive(Debug, Serialize)]
struct ShaderLoadingContext<'a> {
	pub passes: &'a Vec<Pass>,
}

fn main() -> Result<(), String> {
	let config = Config::load().map_err(|err| format!("Failed to load config: {}.", err))?;
	println!("{:?}", config);

	let mut tera = Tera::default();
	tera.add_raw_templates(vec![
		("api", templates::API),
		("blender_api", templates::BLENDER_API),
		("render", templates::RENDER),
		("shader_declarations", templates::SHADER_DECLARATIONS),
		("shader_loading", templates::SHADER_LOADING),
	])
	.expect("Failed to load templates.");

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

	let pass0 = Pass {
		fragment: Some(FRAGMENT.to_string()),
		vertex: None,
	};
	let passes = vec![pass0];

	let empty_context = Context::new();

	let api_context = APIContext {
		blender_api: true,
		development: true,
	};
	let api_contents = tera
		.render("api", &api_context)
		.map_err(|_| "Failed to render template: api.")?;

	let render_contents = tera
		.render("render", &empty_context)
		.map_err(|_| "Failed to render template: render.")?;

	let shader_declarations_context = ShaderDeclarationContext { passes: &passes };
	let shader_declarations_contents = tera
		.render("shader_declarations", &shader_declarations_context)
		.map_err(|err| {
			format!(
				"Failed to render template: shader_declarations {}.",
				err.kind()
			)
		})?;

	let shader_loading_context = ShaderLoadingContext { passes: &passes };
	let shader_loading_contents = tera
		.render("shader_loading", &shader_loading_context)
		.map_err(|err| format!("Failed to render template: shader_loading {}.", err.kind()))?;

	let blender_empty_context = BlenderAPIContext {
		api: &api_contents,
		passes: &passes,
		render: &render_contents,
		shader_declarations: &shader_declarations_contents,
		shader_loading: &shader_loading_contents,
	};
	let blender_api_contents = tera
		.render("blender_api", &blender_empty_context)
		.map_err(|err| format!("Failed to render template: blender_api: {}.", err))?;

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
			PathBuf::from(&config.paths.glew)
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
		.args(&config.link.args)
		.arg(format!(
			"{}",
			PathBuf::from(&config.paths.glew)
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
