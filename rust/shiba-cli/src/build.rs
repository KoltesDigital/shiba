use crate::compiler::{CompileOptions, CompilerKind};
use crate::project_files::{self, ProjectFiles};
use crate::shader_codes::to_standalone_passes;
use crate::types::{CompilationDescriptor, Pass, ProjectDescriptor, Variable};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
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

pub struct ShaderProvidedEvent<'a> {
	pub passes: Vec<Pass>,
	pub variables: &'a Vec<Variable>,
}

pub struct StaticFilesProvidedEvent<'a> {
	pub paths: &'a Vec<PathBuf>,
}

pub enum BuildEvent<'a> {
	ExecutableCompiled(ExecutableCompiledEvent),
	LibraryCompiled(LibraryCompiledEvent),
	ShaderProvided(ShaderProvidedEvent<'a>),
	StaticFilesProvided(StaticFilesProvidedEvent<'a>),
}

#[derive(Clone, Copy, Debug, Deserialize, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BuildTarget {
	Executable,
	Library,
}

impl FromStr for BuildTarget {
	type Err = &'static str;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"executable" => Ok(BuildTarget::Executable),
			"library" => Ok(BuildTarget::Library),
			_ => Err("Invalid target variant."),
		}
	}
}

pub struct BuildOptions<'a> {
	pub force: bool,
	pub project_descriptor: &'a ProjectDescriptor,
	pub target: BuildTarget,
}

pub fn build(
	options: &BuildOptions,
	event_listener: &mut dyn FnMut(BuildEvent) -> (),
) -> Result<(), String> {
	let audio_synthesizer = options
		.project_descriptor
		.settings
		.audio_synthesizer
		.instantiate(&options.project_descriptor)?;

	let shader_minifier = options
		.project_descriptor
		.settings
		.shader_minifier
		.as_ref()
		.map(|shader_minifier| shader_minifier.instantiate(&options.project_descriptor))
		.transpose()?;

	let shader_provider = options
		.project_descriptor
		.settings
		.shader_provider
		.instantiate(&options.project_descriptor)?;

	let compiler = options
		.project_descriptor
		.instantiate_compiler(options.target)?;

	let project_files = ProjectFiles::load(
		&options.project_descriptor.directory,
		&project_files::LoadOptions {
			compiler_paths: &[match compiler {
				CompilerKind::Executable(ref compiler) => compiler.get_is_path_handled(),
				CompilerKind::Library(ref compiler) => compiler.get_is_path_handled(),
			}],
			ignore_paths: &[
				Box::new(|path| {
					if let Some(file_name) = path.file_name() {
						let file_name = file_name.to_string_lossy();
						if file_name.starts_with('.') {
							return true;
						}
						if file_name == "shiba.yml" {
							return true;
						}
					}
					false
				}),
				audio_synthesizer.get_is_path_handled(),
				shader_provider.get_is_path_handled(),
			],
		},
	)?;

	event_listener(BuildEvent::StaticFilesProvided(StaticFilesProvidedEvent {
		paths: project_files.get_static_files(),
	}));

	let project_codes =
		project_files.get_compiler_codes(options.project_descriptor.development, options.target)?;

	let compilation_descriptor = CompilationDescriptor::default();

	let integration_result = audio_synthesizer.integrate(options, &compilation_descriptor)?;
	let audio_codes = integration_result.codes;
	let compilation_descriptor = integration_result.compilation_descriptor;

	let mut shader_descriptor = shader_provider.provide(options)?;

	if let Some(shader_minifier) = shader_minifier {
		shader_descriptor = shader_minifier.minify(options, &shader_descriptor)?;
	}

	event_listener(BuildEvent::ShaderProvided(ShaderProvidedEvent {
		passes: to_standalone_passes(&shader_descriptor),
		variables: &shader_descriptor.variables,
	}));

	let compile_options = CompileOptions {
		audio_codes: &audio_codes,
		compilation_descriptor: &compilation_descriptor,
		project_codes: &project_codes,
		shader_descriptor: &shader_descriptor,
	};

	match compiler {
		CompilerKind::Executable(ref compiler) => {
			let path = compiler.compile(options, &compile_options)?;

			event_listener(BuildEvent::ExecutableCompiled(ExecutableCompiledEvent {
				path,
			}));
		}
		CompilerKind::Library(ref compiler) => {
			let path = compiler.compile(options, &compile_options)?;

			event_listener(BuildEvent::LibraryCompiled(LibraryCompiledEvent { path }));
		}
	};

	Ok(())
}

pub fn build_duration(
	options: &BuildOptions,
	event_listener: &mut dyn FnMut(BuildEvent) -> (),
) -> Result<Duration, String> {
	let start = Instant::now();

	build(options, event_listener)?;

	let duration = start.elapsed();
	Ok(duration)
}
