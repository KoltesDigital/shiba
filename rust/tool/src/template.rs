use serde::Serialize;
use tera::{Context, Tera, Value};

macro_rules! template_enum {
	(
		$($variant:ident => $filename:expr),*,
	) => {
		pub enum Template {
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
	API => "api",
	BlenderAPI => "blender_api",
	Render => "render",
	ShaderDeclarations => "shader_declarations",
	ShaderLoading => "shader_loading",
}

pub struct TemplateRenderer {
	tera: Tera,
}

impl TemplateRenderer {
	pub fn new() -> Result<Self, String> {
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

		Ok(TemplateRenderer { tera })
	}

	pub fn render(&self, template: Template) -> Result<String, String> {
		self.render_context(template, &Context::new())
	}

	pub fn render_context<T: Serialize>(
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
