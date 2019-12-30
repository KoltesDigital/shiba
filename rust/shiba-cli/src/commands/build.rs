use crate::build::{self, BuildEvent, BuildOptions, BuildTarget};
use std::path::Path;

pub struct Options<'a> {
	pub force: bool,
	pub project_directory: &'a Path,
	pub target: BuildTarget,
}

pub fn execute(options: &Options) -> Result<(), String> {
	let event_listener = |event: BuildEvent| match event {
		BuildEvent::ExecutableCompiled(event) => match event.get_size() {
			Ok(size) => {
				println!("Executable compiled:");
				println!("  Path: {:?}", event.path);
				println!("  Size: {}", size);
			}
			Err(_err) => {
				println!("Unexpected error while getting size.");
			}
		},

		BuildEvent::LibraryCompiled(event) => {
			println!("Library compiled:");
			println!("  Path: {:?}", event.path);
		}

		_ => {}
	};

	let duration = build::build_duration(&BuildOptions {
		event_listener: &event_listener,
		force: options.force,
		project_directory: &options.project_directory,
		target: options.target,
	})?;

	println!("Build duration: {:?}.", duration);
	Ok(())
}
