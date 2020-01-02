use crate::build::{self, BuildEvent, BuildOptions, BuildTarget};
use crate::run::{self, RunOptions};
use crate::types::ProjectDescriptor;
use std::cell::Cell;
use std::path::Path;

pub struct Options<'a> {
	pub project_directory: &'a Path,
}

pub fn execute(options: &Options) -> Result<(), String> {
	let project_descriptor =
		ProjectDescriptor::load(options.project_directory, BuildTarget::Executable)?;

	let executable_path = Cell::new(None);

	let event_listener = |event: BuildEvent| {
		if let BuildEvent::ExecutableCompiled(event) = event {
			executable_path.set(Some(event.path));
		}
	};

	build::build(&BuildOptions {
		event_listener: &event_listener,
		force: false,
		project_descriptor: &project_descriptor,
		target: BuildTarget::Executable,
	})?;

	let executable_path = executable_path.take().unwrap();

	run::run(&RunOptions {
		executable_path: &executable_path,
		project_directory: &options.project_directory,
	})?;

	Ok(())
}
