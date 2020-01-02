mod settings;

pub use self::settings::NoneSettings;
use super::{AudioSynthesizer, IntegrationResult};
use crate::build::BuildOptions;
use crate::project_files::{CodeMap, FileConsumer, IsPathHandled};
use crate::types::{CompilationDescriptor, ProjectDescriptor};
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

pub struct NoneAudioSynthesizer<'a> {
	settings: &'a NoneSettings,

	tera: Tera,
}

impl<'a> NoneAudioSynthesizer<'a> {
	pub fn new(
		_project_descriptor: &'a ProjectDescriptor,
		settings: &'a NoneSettings,
	) -> Result<Self, String> {
		let mut tera = Tera::default();

		tera.add_raw_templates(Template::as_array())
			.map_err(|err| err.to_string())?;

		Ok(NoneAudioSynthesizer { settings, tera })
	}
}

impl<'a> AudioSynthesizer for NoneAudioSynthesizer<'a> {
	fn integrate(
		&self,
		_build_options: &BuildOptions,
		compilation_descriptor: &CompilationDescriptor,
	) -> Result<IntegrationResult, String> {
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

		Ok(IntegrationResult {
			codes,
			compilation_descriptor: compilation_descriptor.clone(),
		})
	}
}

impl FileConsumer for NoneAudioSynthesizer<'_> {
	fn get_is_path_handled<'b, 'a: 'b>(&'a self) -> IsPathHandled<'b> {
		Box::new(|_path| false)
	}
}
