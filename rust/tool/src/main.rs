#[macro_use]
extern crate lazy_static;

mod config_provider;
mod directories;
mod generators {
	pub mod blender_api;
}
mod template;
mod types;

use crate::template::TemplateRenderer;
use crate::types::Pass;
use config_provider::ConfigProvider;
use std::str;

static FRAGMENT: &str = include_str!("shader.frag");

fn main() -> Result<(), String> {
	let config = ConfigProvider::load();

	let generator = generators::blender_api::BlenderAPIGenerator::new(&config)?;

	let pass0 = Pass {
		fragment: Some(FRAGMENT.to_string()),
		vertex: None,
	};
	let passes = vec![pass0];

	let template_renderer = TemplateRenderer::new()?;

	generator.generate(&template_renderer, &passes)?;

	Ok(())
}
