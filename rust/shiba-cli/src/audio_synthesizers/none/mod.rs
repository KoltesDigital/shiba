mod settings;

pub use self::settings::NoneSettings;
use super::AudioSynthesizer;
use crate::build::BuildOptions;
use crate::compilation::CompilationJobEmitter;
use crate::compilation_data::Compilation;
use crate::project_data::Project;
use crate::project_files::{CodeMap, FileConsumer, IsPathHandled};
use crate::{Error, Result};
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
	pub fn new(_project: &'a Project, settings: &'a NoneSettings) -> Result<Self> {
		let mut tera = Tera::default();

		tera.add_raw_templates(Template::as_array())
			.expect("Failed to add templates.");

		Ok(NoneAudioSynthesizer { settings, tera })
	}
}

impl<'a> AudioSynthesizer for NoneAudioSynthesizer<'a> {
	fn integrate(
		&self,
		_build_options: &BuildOptions,
		_compilation: &mut Compilation,
	) -> Result<CodeMap> {
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
					&Context::from_serialize(&context).expect("Failed to create context."),
				)
				.map_err(|err| Error::failed_to_render_template(&name, err))?;
			codes.insert(name.to_string(), s);
		}

		Ok(codes)
	}
}

impl CompilationJobEmitter for NoneAudioSynthesizer<'_> {
	fn requires_asm_compiler(&self) -> bool {
		false
	}

	fn requires_cpp_compiler(&self) -> bool {
		false
	}
}

impl FileConsumer for NoneAudioSynthesizer<'_> {
	fn get_is_path_handled<'b, 'a: 'b>(&'a self) -> IsPathHandled<'b> {
		Box::new(|_path| false)
	}
}
