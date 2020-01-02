use crate::build::{self, BuildEvent, BuildOptions, BuildTarget};
use crate::export::{self, ExportOptions, ExportOutput};
use crate::types::ProjectDescriptor;
use std::cell::Cell;
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

	let build_path = Cell::new(None);

	let event_listener = |event: BuildEvent| match event {
		BuildEvent::ExecutableCompiled(event) => {
			build_path.set(Some(event.path));
		}

		BuildEvent::LibraryCompiled(event) => {
			build_path.set(Some(event.path));
		}

		_ => (),
	};

	build::build(&BuildOptions {
		event_listener: &event_listener,
		force: options.force,
		project_descriptor: &project_descriptor,
		target: options.target,
	})?;

	let build_path = build_path.take().unwrap();

	export::export(&ExportOptions {
		build_path: &build_path,
		directory: options.export_directory,
		project_descriptor: &project_descriptor,
		output: options.output,
	})?;

	Ok(())
}
