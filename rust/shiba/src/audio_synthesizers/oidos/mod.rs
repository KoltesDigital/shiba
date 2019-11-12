mod settings;

pub use self::settings::Settings;
use crate::configuration::Configuration;
use crate::custom_codes::CustomCodes;
use crate::paths::TEMP_DIRECTORY;
use crate::traits;
use crate::types::CompilationDescriptor;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Command;
use tera::Tera;

template_enum! {
	audio_duration: "audio_duration",
	audio_is_playing: "audio_is_playing",
	audio_start: "audio_start",
	audio_time: "audio_time",
	declarations: "declarations",
}

#[derive(Serialize)]
struct Context {}

pub struct AudioSynthesizer<'a> {
	nasm_path: PathBuf,
	oidos_path: PathBuf,
	project_directory: &'a Path,
	python2_path: PathBuf,
	settings: &'a Settings,
	tera: Tera,
}

impl<'a> AudioSynthesizer<'a> {
	pub fn new(
		project_directory: &'a Path,
		settings: &'a Settings,
		configuration: &Configuration,
	) -> Result<Self, String> {
		let nasm_path = configuration
			.paths
			.get("nasm")
			.cloned()
			.unwrap_or_else(|| PathBuf::from("nasm"));

		let oidos_path = configuration
			.paths
			.get("oidos")
			.ok_or("Please set configuration key paths.oidos.")?
			.clone();

		let python2_path = configuration
			.paths
			.get("python2")
			.cloned()
			.unwrap_or_else(|| PathBuf::from("python"));

		let mut tera = Tera::default();

		tera.add_raw_templates(Template::as_array())
			.map_err(|err| err.to_string())?;

		Ok(AudioSynthesizer {
			nasm_path,
			oidos_path,
			project_directory,
			python2_path,
			tera,
			settings,
		})
	}
}

impl<'a> traits::AudioSynthesizer for AudioSynthesizer<'a> {
	fn integrate(
		&self,
		custom_codes: &mut CustomCodes,
		compilation_descriptor: &mut CompilationDescriptor,
	) -> Result<(), String> {
		let mut conversion = Command::new(&self.python2_path)
			.arg(
				self.oidos_path
					.join("convert")
					.join("OidosConvert.py")
					.to_string_lossy()
					.as_ref(),
			)
			.arg(
				self.project_directory
					.join(self.settings.filename.as_str())
					.to_string_lossy()
					.as_ref(),
			)
			.arg(TEMP_DIRECTORY.join("music.asm").to_string_lossy().as_ref())
			.spawn()
			.map_err(|err| err.to_string())?;

		let status = conversion.wait().map_err(|err| err.to_string())?;
		if !status.success() {
			return Err("Failed to convert music.".to_string());
		}

		for (output, input) in &[
			("oidos.obj", "oidos.asm"),
			("oidos-random.obj", "random.asm"),
		] {
			let mut compilation = Command::new(&self.nasm_path)
				.args(vec!["-f", "win32", "-i"])
				.arg(&*TEMP_DIRECTORY)
				.arg("-i")
				.arg(&self.project_directory.to_string_lossy().as_ref())
				.arg("-o")
				.arg(output)
				.arg(
					self.oidos_path
						.join("player")
						.join(input)
						.to_string_lossy()
						.as_ref(),
				)
				.current_dir(&*TEMP_DIRECTORY)
				.spawn()
				.map_err(|err| err.to_string())?;

			let status = compilation.wait().map_err(|err| err.to_string())?;
			if !status.success() {
				return Err("Failed to compile.".to_string());
			}
		}

		let context = Context {};

		for (name, _) in Template::as_array() {
			let s = self
				.tera
				.render(name, &context)
				.map_err(|_| format!("Failed to render {}.", name))?;
			*custom_codes
				.entry(name.to_string())
				.or_insert_with(String::new) += s.as_str();
		}

		compilation_descriptor.cl.args.push(format!(
			"/I{}",
			self.oidos_path.join("player").to_string_lossy()
		));

		compilation_descriptor
			.crinkler
			.args
			.push("winmm.lib".to_string());
		compilation_descriptor
			.crinkler
			.args
			.push("oidos.obj".to_string());
		compilation_descriptor
			.crinkler
			.args
			.push("oidos-random.obj".to_string());

		Ok(())
	}
}
