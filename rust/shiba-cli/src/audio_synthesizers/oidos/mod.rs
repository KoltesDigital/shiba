mod settings;

pub use self::settings::OidosSettings;
use super::{AudioSynthesizer, IntegrationResult};
use crate::code_map::CodeMap;
use crate::hash_extra;
use crate::paths::BUILD_ROOT_DIRECTORY;
use crate::types::{CompilationDescriptor, ProjectDescriptor};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tera::Tera;

template_enum! {
	declarations: "declarations",
	duration: "duration",
	initialization: "initialization",
	is_playing: "is_playing",
	time_definition: "time_definition",
}

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

const OUTPUT_FILENAME: &str = "integration-result.json";

#[derive(Hash)]
struct Inputs<'a> {
	nasm_path: &'a Path,
	oidos_path: &'a Path,
	python2_path: &'a Path,
	settings: &'a OidosSettings,
}

#[derive(Deserialize)]
struct Outputs {}

#[derive(Serialize)]
struct Context {}

impl<'a> AudioSynthesizer for OidosAudioSynthesizer<'a> {
	fn integrate(
		&self,
		compilation_descriptor: &CompilationDescriptor,
	) -> Result<IntegrationResult, String> {
		let inputs = Inputs {
			nasm_path: &self.nasm_path,
			oidos_path: &self.oidos_path,
			python2_path: &self.python2_path,
			settings: self.settings,
		};
		let build_cache_directory = hash_extra::get_build_cache_directory(&inputs)?;
		let build_cache_path = build_cache_directory.join(OUTPUT_FILENAME);

		if build_cache_path.exists() {
			let json = fs::read_to_string(build_cache_path).map_err(|err| err.to_string())?;
			let integration_result =
				serde_json::from_str(json.as_str()).map_err(|_| "Failed to parse JSON.")?;
			return Ok(integration_result);
		}

		let build_directory = BUILD_ROOT_DIRECTORY
			.join("audio-synthesizers")
			.join("oidos");
		fs::create_dir_all(&build_directory).map_err(|err| err.to_string())?;

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
			.arg(build_directory.join("music.asm").to_string_lossy().as_ref())
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
				.arg(&*build_directory)
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
				.current_dir(&*build_directory)
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

		let mut compilation_descriptor = compilation_descriptor.clone();

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

		let integration_result = IntegrationResult {
			codes,
			compilation_descriptor,
		};

		let json =
			serde_json::to_string(&integration_result).map_err(|_| "Failed to dump JSON.")?;
		fs::write(build_cache_path, json).map_err(|err| err.to_string())?;

		Ok(integration_result)
	}
}
