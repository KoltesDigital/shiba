mod settings;

pub use self::settings::NoneSettings;
use super::{AudioSynthesizer, IntegrationResult};
use crate::build::BuildOptions;
use crate::compilation_data::Compilation;
use crate::project_data::Project;
use crate::project_files::{CodeMap, FileConsumer, IsPathHandled};
use ordered_float::OrderedFloat;
use serde::Serialize;
use tera::{Context, Tera};

template_enum! {
	declarations: "declarations",
	duration: "duration",
	initialization: "initialization",
	is_playing: "is_playing",
	time_definition: "time_definition",
}

pub struct NoneAudioSynthesizer<'a> {
	settings: &'a NoneSettings,

	tera: Tera,
}

impl<'a> NoneAudioSynthesizer<'a> {
	pub fn new(_project: &'a Project, settings: &'a NoneSettings) -> Result<Self, String> {
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
		compilation: &Compilation,
	) -> Result<IntegrationResult, String> {
		#[derive(Serialize)]
		struct OwnContext {
			speed: Option<OrderedFloat<f32>>,
		}

		let context = OwnContext {
			speed: self.settings.speed,
		};

		let mut codes = CodeMap::default();
		for (name, _) in Template::as_array() {
			let s = self
				.tera
				.render(
					name,
					&Context::from_serialize(&context).map_err(|err| err.to_string())?,
				)
				.map_err(|err| err.to_string())?;
			codes.insert(name.to_string(), s);
		}

		Ok(IntegrationResult {
			codes,
			compilation: compilation.clone(),
		})
	}
}

impl FileConsumer for NoneAudioSynthesizer<'_> {
	fn get_is_path_handled<'b, 'a: 'b>(&'a self) -> IsPathHandled<'b> {
		Box::new(|_path| false)
	}
}
