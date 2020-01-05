use crate::build::{self, BuildEvent, BuildOptions, BuildTarget};
use crate::export::{self, ExportOptions, ExportOutput};
use crate::project_data::Project;
use std::path::Path;

pub struct Options<'a> {
	pub export_directory: &'a Path,
	pub force: bool,
	pub output: ExportOutput,
	pub project_directory: &'a Path,
	pub target: BuildTarget,
}

pub fn execute(options: &Options) -> Result<(), String> {
	let project = Project::load(options.project_directory, options.target)?;

	let mut build_path = None;
	let mut static_files = None;

	let mut event_listener = |event: BuildEvent| match event {
		BuildEvent::ExecutableBuilt(event) => {
			build_path = Some(event.path.to_path_buf());
		}

		BuildEvent::LibraryBuilt(event) => {
			build_path = Some(event.path.to_path_buf());
		}

		BuildEvent::StaticFilesProvided(event) => {
			static_files = Some(event.paths.clone());
		}

		_ => (),
	};

	build::build(
		&BuildOptions {
			force: options.force,
			project: &project,
			target: options.target,
		},
		&mut event_listener,
	)?;

	export::export(&ExportOptions {
		build_path: &build_path.unwrap(),
		directory: options.export_directory,
		project: &project,
		output: options.output,
		static_files: &static_files.unwrap(),
	})?;

	Ok(())
}
