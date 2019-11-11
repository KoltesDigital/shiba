mod settings;

pub use self::settings::Settings;
use crate::custom_codes::CustomCodes;
use crate::traits;
use ordered_float::OrderedFloat;
use serde::Serialize;
use tera::Tera;

template_enum! {
	audio_duration: "audio_duration",
	audio_is_playing: "audio_is_playing",
	audio_start: "audio_start",
	audio_time: "audio_time",
	declarations: "declarations",
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
	fn integrate(&self, custom_codes: &mut CustomCodes) -> Result<(), String> {
		let context = Context {
			speed: self.settings.speed,
		};

		for (name, _) in Template::as_array() {
			let s = self
				.tera
				.render(name, &context)
				.map_err(|_| format!("Failed to render {}.", name))?;
			*custom_codes
				.entry(name.to_string())
				.or_insert_with(String::new) += s.as_str();
		}

		Ok(())
	}
}
