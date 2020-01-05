use crate::build::{self, BuildEvent, BuildOptions, BuildTarget};
use crate::project_data::Project;
use crate::run::{self, RunOptions};
use std::path::Path;

pub struct Options<'a> {
	pub project_directory: &'a Path,
}

pub fn execute(options: &Options) -> Result<(), String> {
	let project = Project::load(options.project_directory, BuildTarget::Executable)?;

	let mut executable_path = None;

	let mut event_listener = |event: BuildEvent| {
		if let BuildEvent::ExecutableBuilt(event) = event {
			executable_path = Some(event.path.to_path_buf());
		}
	};

	build::build(
		&BuildOptions {
			force: false,
			project: &project,
			target: BuildTarget::Executable,
		},
		&mut event_listener,
	)?;

	run::run(&RunOptions {
		executable_path: &executable_path.unwrap(),
		project_directory: &options.project_directory,
	})?;

	Ok(())
}
