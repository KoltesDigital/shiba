use crate::build::{self, BuildEvent, BuildOptions, BuildTarget};
use crate::run::{self, RunOptions};
use crate::types::ProjectDescriptor;
use std::path::Path;

pub struct Options<'a> {
	pub project_directory: &'a Path,
}

pub fn execute(options: &Options) -> Result<(), String> {
	let project_descriptor =
		ProjectDescriptor::load(options.project_directory, BuildTarget::Executable)?;

	let mut executable_path = None;

	let mut event_listener = |event: BuildEvent| {
		if let BuildEvent::ExecutableCompiled(event) = event {
			executable_path = Some(event.path);
		}
	};

	build::build(
		&BuildOptions {
			force: false,
			project_descriptor: &project_descriptor,
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
