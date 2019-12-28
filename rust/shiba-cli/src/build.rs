use crate::code_map;
use crate::compiler::{CompileOptions, CompilerKind};
use crate::shader_codes::to_standalone_passes;
use crate::types::{CompilationDescriptor, Pass, ProjectDescriptor};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub struct ExecutableCompiledEvent {
	pub path: PathBuf,
}

impl ExecutableCompiledEvent {
	pub fn get_size(&self) -> Result<u64, String> {
		let size = fs::metadata(&self.path)
			.map_err(|err| err.to_string())?
			.len();
		Ok(size)
	}
}

pub struct LibraryCompiledEvent {
	pub path: PathBuf,
}

pub struct ShaderPassesGeneratedEvent {
	pub passes: Vec<Pass>,
}

pub enum BuildEvent {
	ExecutableCompiled(ExecutableCompiledEvent),
	LibraryCompiled(LibraryCompiledEvent),
	ShaderPassesGenerated(ShaderPassesGeneratedEvent),
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BuildMode {
	/// Always build everything.
	Force,
	/// If something changed, build everything.
	Full,
	/// If something changed, build that only.
	Updates,
}

impl BuildMode {
	pub fn for_command(force: bool) -> Self {
		if force {
			BuildMode::Force
		} else {
			BuildMode::Full
		}
	}

	pub fn should_emit_compiled_events(self) -> bool {
		self == BuildMode::Force || self == BuildMode::Full
	}
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BuildTarget {
	Executable,
	Library,
}

pub struct BuildOptions<'a> {
	pub event_listener: &'a dyn Fn(BuildEvent) -> (),
	pub mode: BuildMode,
	pub project_directory: &'a Path,
	pub target: BuildTarget,
}

pub fn build(options: &BuildOptions) -> Result<(), String> {
	let project_descriptor = ProjectDescriptor::load(options)?;

	let audio_synthesizer = project_descriptor
		.settings
		.audio_synthesizer
		.instantiate(&project_descriptor)?;

	let shader_minifier = project_descriptor
		.settings
		.shader_minifier
		.as_ref()
		.map(|shader_minifier| shader_minifier.instantiate(&project_descriptor))
		.transpose()?;

	let shader_provider = project_descriptor
		.settings
		.shader_provider
		.instantiate(&project_descriptor)?;

	let compiler = project_descriptor.instantiate_compiler()?;

	let project_codes =
		code_map::load_project_codes(options.project_directory, project_descriptor.development)?;
	/*
	let build_hash_path = LOCAL_DATA_DIRECTORY.join("executable.build.hash");
	let mut build_hash = StoredHash::new(&build_hash_path);

	{
		let mut updater = build_hash.get_updater();
		updater.add(&project_descriptor);
		updater.add(&project_codes);
		// -updater.add(&shader_provider);
	}

	let must_build = options.mode == BuildMode::Force
		|| build_hash.has_changed()
		|| !options.compiler.get_path().exists();
	if must_build {
		*/
	let mut compilation_descriptor = CompilationDescriptor::default();

	let audio_codes = audio_synthesizer.integrate(&mut compilation_descriptor)?;

	let mut shader_descriptor = shader_provider.provide()?;

	if let Some(shader_minifier) = shader_minifier {
		shader_descriptor = shader_minifier.minify(&shader_descriptor)?;
	}

	(project_descriptor.build_options.event_listener)(BuildEvent::ShaderPassesGenerated(
		ShaderPassesGeneratedEvent {
			passes: to_standalone_passes(&shader_descriptor),
		},
	));

	let compile_options = CompileOptions {
		audio_codes: &audio_codes,
		compilation_descriptor: &compilation_descriptor,
		project_codes: &project_codes,
		shader_descriptor: &shader_descriptor,
	};

	match compiler {
		CompilerKind::Executable(compiler) => {
			let path = compiler.compile(&compile_options)?;

			if project_descriptor
				.build_options
				.mode
				.should_emit_compiled_events()
			{
				(project_descriptor.build_options.event_listener)(BuildEvent::ExecutableCompiled(
					ExecutableCompiledEvent { path },
				));
			}
		}
		CompilerKind::Library(compiler) => {
			let path = compiler.compile(&compile_options)?;

			if project_descriptor
				.build_options
				.mode
				.should_emit_compiled_events()
			{
				(project_descriptor.build_options.event_listener)(BuildEvent::LibraryCompiled(
					LibraryCompiledEvent { path },
				));
			}
		}
	};

	//let _ = build_hash.store();

	Ok(())
}

pub fn build_duration(options: &BuildOptions) -> Result<Duration, String> {
	let start = Instant::now();

	build(options)?;

	let duration = start.elapsed();

	Ok(duration)
}
