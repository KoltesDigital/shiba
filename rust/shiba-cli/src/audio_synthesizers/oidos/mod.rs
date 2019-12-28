mod settings;

pub use self::settings::OidosSettings;
use super::AudioSynthesizer;
use crate::code_map::CodeMap;
use crate::paths::TEMP_DIRECTORY;
use crate::types::{CompilationDescriptor, ProjectDescriptor};
use serde::Serialize;
use std::path::PathBuf;
use std::process::Command;
use tera::Tera;

template_enum! {
	declarations: "declarations",
	duration: "duration",
	initialization: "initialization",
	is_playing: "is_playing",
	time_definition: "time_definition",
}

#[derive(Serialize)]
struct Context {}

pub struct OidosAudioSynthesizer<'a> {
	project_descriptor: &'a ProjectDescriptor<'a>,
	settings: &'a OidosSettings,

	nasm_path: PathBuf,
	oidos_path: PathBuf,
	python2_path: PathBuf,
	tera: Tera,
}

impl<'a> OidosAudioSynthesizer<'a> {
	pub fn new(
		project_descriptor: &'a ProjectDescriptor,
		settings: &'a OidosSettings,
	) -> Result<Self, String> {
		let nasm_path = project_descriptor
			.configuration
			.paths
			.get("nasm")
			.cloned()
			.unwrap_or_else(|| PathBuf::from("nasm"));

		let oidos_path = project_descriptor
			.configuration
			.paths
			.get("oidos")
			.ok_or("Please set project_descriptor.configuration key paths.oidos.")?
			.clone();

		let python2_path = project_descriptor
			.configuration
			.paths
			.get("python2")
			.cloned()
			.unwrap_or_else(|| PathBuf::from("python"));

		let mut tera = Tera::default();

		tera.add_raw_templates(Template::as_array())
			.map_err(|err| err.to_string())?;

		Ok(OidosAudioSynthesizer {
			project_descriptor,
			settings,

			nasm_path,
			oidos_path,
			python2_path,
			tera,
		})
	}
}

impl<'a> AudioSynthesizer for OidosAudioSynthesizer<'a> {
	fn integrate(
		&self,
		compilation_descriptor: &mut CompilationDescriptor,
	) -> Result<CodeMap, String> {
		let mut conversion = Command::new(&self.python2_path)
			.arg(
				self.oidos_path
					.join("convert")
					.join("OidosConvert.py")
					.to_string_lossy()
					.as_ref(),
			)
			.arg(
				self.project_descriptor
					.build_options
					.project_directory
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
				.arg(
					&self
						.project_descriptor
						.build_options
						.project_directory
						.to_string_lossy()
						.as_ref(),
				)
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

		let mut codes = CodeMap::default();
		for (name, _) in Template::as_array() {
			let s = self
				.tera
				.render(name, &context)
				.map_err(|_| format!("Failed to render {}.", name))?;
			codes.insert(name.to_string(), s);
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

		Ok(codes)
	}
}
