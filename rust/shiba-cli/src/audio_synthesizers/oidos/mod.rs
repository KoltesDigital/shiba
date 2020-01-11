mod settings;

pub use self::settings::OidosSettings;
use super::AudioSynthesizer;
use crate::build::BuildOptions;
use crate::compilation::CompilationJobEmitter;
use crate::compilation_data::{Compilation, CompilationJob, CompilationJobKind};
use crate::hash_extra;
use crate::paths::BUILD_ROOT_DIRECTORY;
use crate::project_data::Project;
use crate::project_files::{CodeMap, FileConsumer, IsPathHandled};
use crate::{Error, Result};
use serde::Serialize;
use serde_json;
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tera::{Context, Tera};

template_enum! {
	declarations: "declarations",
	duration: "duration",
	initialization: "initialization",
	is_playing: "is_playing",
	time_definition: "time_definition",
}

pub struct OidosAudioSynthesizer<'a> {
	settings: &'a OidosSettings,

	oidos_path: PathBuf,
	python2_path: PathBuf,
	tera: Tera,
}

impl<'a> OidosAudioSynthesizer<'a> {
	pub fn new(project: &'a Project, settings: &'a OidosSettings) -> Result<Self> {
		let oidos_path = project.configuration.get_path("oidos");

		let python2_path = project.configuration.get_path("python2");

		let mut tera = Tera::default();

		tera.add_raw_templates(Template::as_array())
			.expect("Failed to add templates.");

		Ok(OidosAudioSynthesizer {
			settings,

			oidos_path,
			python2_path,
			tera,
		})
	}
}

impl<'a> AudioSynthesizer for OidosAudioSynthesizer<'a> {
	fn integrate(
		&self,
		build_options: &BuildOptions,
		compilation: &mut Compilation,
	) -> Result<CodeMap> {
		const OUTPUT_FILENAME: &str = "codes.json";

		let mut path = Cow::from(&self.settings.path);
		if path.is_relative() {
			path = Cow::from(build_options.project.directory.join(path));
		}

		let contents = fs::read(&path).map_err(|err| Error::failed_to_read(&path.as_ref(), err))?;

		#[derive(Hash)]
		struct Inputs<'a> {
			contents: &'a [u8],
			oidos_path: &'a Path,
			python2_path: &'a Path,
			settings: &'a OidosSettings,
		}

		let inputs = Inputs {
			contents: &contents,
			oidos_path: &self.oidos_path,
			python2_path: &self.python2_path,
			settings: self.settings,
		};
		let build_cache_directory = hash_extra::get_build_cache_directory(&inputs)?;
		let build_cache_path = build_cache_directory.join(OUTPUT_FILENAME);

		compilation
			.include_paths
			.insert(self.oidos_path.join("player"));
		compilation
			.common
			.link_library_paths
			.insert(build_cache_directory.clone());
		compilation
			.common
			.link_dependencies
			.insert(PathBuf::from("winmm.lib"));

		let mut include_paths = BTreeSet::new();
		include_paths.insert(build_cache_directory.clone());

		for path in &["oidos.asm", "random.asm"] {
			compilation.jobs.push(CompilationJob {
				kind: CompilationJobKind::Asm,
				path: self.oidos_path.join("player").join(path),
				include_paths: include_paths.clone(),
			});
		}

		if !build_options.force && build_cache_path.exists() {
			let json = fs::read_to_string(&build_cache_path)
				.map_err(|err| Error::failed_to_read(&build_cache_path, err))?;
			let codes = serde_json::from_str(&json)
				.map_err(|err| Error::failed_to_deserialize(&json, err))?;
			return Ok(codes);
		}

		let build_directory = BUILD_ROOT_DIRECTORY
			.join("audio-synthesizers")
			.join("oidos");
		fs::create_dir_all(&build_directory)
			.map_err(|err| Error::failed_to_create_directory(&build_directory, err))?;

		let mut conversion = Command::new(&self.python2_path)
			.arg(
				self.oidos_path
					.join("convert")
					.join("OidosConvert.py")
					.to_string_lossy()
					.as_ref(),
			)
			.arg(path.to_string_lossy().as_ref())
			.arg("music.asm")
			.current_dir(&build_directory)
			.spawn()
			.map_err(|err| Error::failed_to_execute(&self.python2_path, err))?;

		let status = conversion.wait().unwrap();
		if !status.success() {
			return Err(Error::execution_failed(&self.python2_path));
		}

		let copy_from = build_directory.join("music.asm");
		let copy_to = build_cache_directory.join("music.asm");
		fs::copy(&copy_from, &copy_to)
			.map_err(|err| Error::failed_to_copy(&copy_from, &copy_to, err))?;

		#[derive(Serialize)]
		struct OwnContext {}

		let context = OwnContext {};

		let mut codes = CodeMap::default();
		for (name, _) in Template::as_array() {
			let s = self
				.tera
				.render(
					&name,
					&Context::from_serialize(&context).expect("Failed to create context."),
				)
				.map_err(|err| Error::failed_to_render_template(&name, err))?;
			codes.insert(name.to_string(), s);
		}

		let json = serde_json::to_string(&codes).expect("Failed to dump JSON.");
		fs::write(&build_cache_path, json)
			.map_err(|err| Error::failed_to_write(&build_cache_path, err))?;

		Ok(codes)
	}
}

impl CompilationJobEmitter for OidosAudioSynthesizer<'_> {
	fn requires_asm_compiler(&self) -> bool {
		true
	}

	fn requires_cpp_compiler(&self) -> bool {
		false
	}
}

impl FileConsumer for OidosAudioSynthesizer<'_> {
	fn get_is_path_handled<'b, 'a: 'b>(&'a self) -> IsPathHandled<'b> {
		Box::new(move |path| path == self.settings.path)
	}
}
