mod settings;

pub use self::settings::Settings;
use crate::code_map::CodeMap;
use crate::traits;
use crate::types::CompilationDescriptor;
use ordered_float::OrderedFloat;
use serde::Serialize;
use tera::Tera;

template_enum! {
	declarations: "declarations",
	duration: "duration",
	initialization: "initialization",
	is_playing: "is_playing",
	time_definition: "time_definition",
}

#[derive(Serialize)]
struct Context {
	speed: Option<OrderedFloat<f32>>,
}

pub struct AudioSynthesizer<'a> {
	settings: &'a Settings,
	tera: Tera,
}

impl<'a> AudioSynthesizer<'a> {
	pub fn new(settings: &'a Settings) -> Result<Self, String> {
		let mut tera = Tera::default();

		tera.add_raw_templates(Template::as_array())
			.map_err(|err| err.to_string())?;

		Ok(AudioSynthesizer { tera, settings })
	}
}

impl<'a> traits::AudioSynthesizer for AudioSynthesizer<'a> {
	fn integrate(
		&self,
		_compilation_descriptor: &mut CompilationDescriptor,
	) -> Result<CodeMap, String> {
		let context = Context {
			speed: self.settings.speed,
		};

		let mut codes = CodeMap::default();
		for (name, _) in Template::as_array() {
			let s = self
				.tera
				.render(name, &context)
				.map_err(|_| format!("Failed to render {}.", name))?;
			codes.insert(name.to_string(), s);
		}

		Ok(codes)
	}
}
