use crate::build::{self, BuildEvent, BuildOptions, BuildTarget};
use crate::export::{self, ExportOptions, ExportOutput};
use crate::types::ProjectDescriptor;
use std::path::Path;

pub struct Options<'a> {
	pub export_directory: &'a Path,
	pub force: bool,
	pub output: ExportOutput,
	pub project_directory: &'a Path,
	pub target: BuildTarget,
}

pub fn execute(options: &Options) -> Result<(), String> {
	let project_descriptor = ProjectDescriptor::load(options.project_directory, options.target)?;

	let mut build_path = None;
	let mut static_files = None;

	let mut event_listener = |event: BuildEvent| match event {
		BuildEvent::ExecutableCompiled(event) => {
			build_path = Some(event.path);
		}

		BuildEvent::LibraryCompiled(event) => {
			build_path = Some(event.path);
		}

		BuildEvent::StaticFilesProvided(event) => {
			static_files = Some(event.paths.clone());
		}

		_ => (),
	};

	build::build(
		&BuildOptions {
			force: options.force,
			project_descriptor: &project_descriptor,
			target: options.target,
		},
		&mut event_listener,
	)?;

	export::export(&ExportOptions {
		build_path: &build_path.unwrap(),
		directory: options.export_directory,
		project_descriptor: &project_descriptor,
		output: options.output,
		static_files: &static_files.unwrap(),
	})?;

	Ok(())
}