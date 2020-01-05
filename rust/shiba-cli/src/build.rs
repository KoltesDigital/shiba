use crate::compilation_data::{Compilation, CompilationJobKind, Linking};
use crate::compilers::CompileOptions;
use crate::linkers::LinkOptions;
use crate::project_data::Project;
use crate::project_files::{self, ProjectFiles};
use crate::shader_data::ShaderSet;
use crate::target_code_generators::{self, GenerateTargetCodeOptions, TargetCodeGenerator};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::{Duration, Instant};

pub struct ExecutableBuiltEvent<'a> {
	pub path: &'a Path,
}

impl ExecutableBuiltEvent<'_> {
	pub fn get_size(&self) -> Result<u64, String> {
		let size = fs::metadata(&self.path)
			.map_err(|err| err.to_string())?
			.len();
		Ok(size)
	}
}

pub struct LibraryBuiltEvent<'a> {
	pub path: &'a Path,
}

pub struct ShaderSetProvidedEvent<'a> {
	pub shader_set: &'a ShaderSet,
}

pub struct StaticFilesProvidedEvent<'a> {
	pub paths: &'a Vec<PathBuf>,
}

pub enum BuildEvent<'a> {
	ExecutableBuilt(ExecutableBuiltEvent<'a>),
	LibraryBuilt(LibraryBuiltEvent<'a>),
	ShaderSetProvided(ShaderSetProvidedEvent<'a>),
	StaticFilesProvided(StaticFilesProvidedEvent<'a>),
}

#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
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
	pub project: &'a Project,
	pub target: BuildTarget,
}

pub fn build(
	options: &BuildOptions,
	event_listener: &mut dyn FnMut(BuildEvent) -> (),
) -> Result<(), String> {
	let audio_synthesizer = options
		.project
		.settings
		.audio_synthesizer
		.instantiate(&options.project, options.target)?;

	let linker = match options.target {
		BuildTarget::Executable => options
			.project
			.settings
			.executable_linker
			.instantiate(options.project)?,
		BuildTarget::Library => options
			.project
			.settings
			.library_linker
			.instantiate(options.project)?,
	};

	let shader_minifier = options
		.project
		.settings
		.shader_minifier
		.as_ref()
		.map(|shader_minifier| shader_minifier.instantiate(&options.project))
		.transpose()?;

	let shader_provider = options
		.project
		.settings
		.shader_provider
		.instantiate(&options.project)?;

	let target_code_generator: Box<(dyn TargetCodeGenerator)> = match options.target {
		BuildTarget::Executable => Box::new(
			target_code_generators::executable::ExecutableTargetCodeGenerator::new(
				&options.project,
			)?,
		),
		BuildTarget::Library => Box::new(
			target_code_generators::library::LibraryTargetCodeGenerator::new(&options.project)?,
		),
	};

	let mut possible_platforms = linker.get_possible_platforms().clone();

	let asm_compiler = if audio_synthesizer.requires_asm_compiler()
		|| target_code_generator.requires_asm_compiler()
	{
		let compiler = options
			.project
			.settings
			.asm_compiler
			.instantiate(&options.project)?;
		possible_platforms = possible_platforms
			.intersection(compiler.get_possible_platforms())
			.cloned()
			.collect();
		Some(compiler)
	} else {
		None
	};

	let cpp_compiler = if audio_synthesizer.requires_cpp_compiler()
		|| target_code_generator.requires_cpp_compiler()
	{
		let compiler = options
			.project
			.settings
			.cpp_compiler
			.instantiate(&options.project)?;
		possible_platforms = possible_platforms
			.intersection(compiler.get_possible_platforms())
			.cloned()
			.collect();
		Some(compiler)
	} else {
		None
	};

	let platform = *possible_platforms
		.iter()
		.next()
		.ok_or("Possible platforms do not intersect.")?;

	let project_files = ProjectFiles::load(
		&options.project.directory,
		&project_files::LoadOptions {
			compiler_paths: &[Box::new(|path| {
				if let Some(extension) = path.extension() {
					if extension.to_string_lossy() == "cpp" {
						return true;
					}
				}
				false
			})],
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
		project_files.get_compiler_codes(options.project.development, options.target)?;

	let mut compilation = Compilation::default();

	let audio_codes = audio_synthesizer.integrate(options, &mut compilation)?;

	let mut shader_set = shader_provider.provide(options)?;

	if let Some(shader_minifier) = shader_minifier {
		shader_set = shader_minifier.minify(options, &shader_set)?;
	}

	event_listener(BuildEvent::ShaderSetProvided(ShaderSetProvidedEvent {
		shader_set: &shader_set,
	}));

	let generate_options = GenerateTargetCodeOptions {
		audio_codes: &audio_codes,
		platform,
		project_codes: &project_codes,
		shader_set: &shader_set,
	};
	target_code_generator.generate(options, &generate_options, &mut compilation)?;

	let mut linking = Linking {
		common: compilation.common,
		..Default::default()
	};

	for mut compilation_job in compilation.jobs.into_iter() {
		let mut include_paths = compilation.include_paths.clone();
		include_paths.append(&mut compilation_job.include_paths);

		let compile_options = CompileOptions {
			audio_codes: &audio_codes,
			include_paths: &include_paths,
			path: &compilation_job.path,
			platform,
			project_codes: &project_codes,
			shader_set: &shader_set,
		};

		match compilation_job.kind {
			CompilationJobKind::Asm => {
				asm_compiler
					.as_ref()
					.unwrap()
					.compile(options, &compile_options, &mut linking)?
			}
			CompilationJobKind::Cpp => {
				cpp_compiler
					.as_ref()
					.unwrap()
					.compile(options, &compile_options, &mut linking)?
			}
		};
	}

	let link_options = LinkOptions {
		linking: &linking,
		platform,
	};
	let path = linker.link(&options, &link_options)?;

	match options.target {
		BuildTarget::Executable => {
			event_listener(BuildEvent::ExecutableBuilt(ExecutableBuiltEvent {
				path: &path,
			}));
		}
		BuildTarget::Library => {
			event_listener(BuildEvent::LibraryBuilt(LibraryBuiltEvent { path: &path }));
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
