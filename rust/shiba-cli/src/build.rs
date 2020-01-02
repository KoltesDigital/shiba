use crate::code_map;
use crate::compiler::{CompileOptions, CompilerKind};
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
	pub target: BuildTarget,
	pub variables: &'a Vec<Variable>,
}

pub enum BuildEvent<'a> {
	ExecutableCompiled(ExecutableCompiledEvent),
	LibraryCompiled(LibraryCompiledEvent),
	ShaderProvided(ShaderProvidedEvent<'a>),
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
	pub event_listener: &'a dyn Fn(BuildEvent) -> (),
	pub force: bool,
	pub project_descriptor: &'a ProjectDescriptor<'a>,
	pub target: BuildTarget,
}

pub fn build(options: &BuildOptions) -> Result<(), String> {
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

	let project_codes = code_map::load_project_codes(
		options.project_descriptor.directory,
		options.project_descriptor.development,
		options.target,
	)?;

	let compilation_descriptor = CompilationDescriptor::default();

	let integration_result = audio_synthesizer.integrate(options, &compilation_descriptor)?;
	let audio_codes = integration_result.codes;
	let compilation_descriptor = integration_result.compilation_descriptor;

	let mut shader_descriptor = shader_provider.provide(options)?;

	if let Some(shader_minifier) = shader_minifier {
		shader_descriptor = shader_minifier.minify(options, &shader_descriptor)?;
	}

	(options.event_listener)(BuildEvent::ShaderProvided(ShaderProvidedEvent {
		passes: to_standalone_passes(&shader_descriptor),
		target: options.target,
		variables: &shader_descriptor.variables,
	}));

	let compile_options = CompileOptions {
		audio_codes: &audio_codes,
		compilation_descriptor: &compilation_descriptor,
		project_codes: &project_codes,
		shader_descriptor: &shader_descriptor,
	};

	match compiler {
		CompilerKind::Executable(compiler) => {
			let path = compiler.compile(options, &compile_options)?;

			(options.event_listener)(BuildEvent::ExecutableCompiled(ExecutableCompiledEvent {
				path,
			}));
		}
		CompilerKind::Library(compiler) => {
			let path = compiler.compile(options, &compile_options)?;

			(options.event_listener)(BuildEvent::LibraryCompiled(LibraryCompiledEvent { path }));
		}
	};

	Ok(())
}

pub fn build_duration(options: &BuildOptions) -> Result<Duration, String> {
	let start = Instant::now();

	build(options)?;

	let duration = start.elapsed();
	Ok(duration)
}
