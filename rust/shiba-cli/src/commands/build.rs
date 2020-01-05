use crate::build::{self, BuildEvent, BuildOptions, BuildTarget};
use crate::project_data::Project;
use std::path::Path;

pub struct Options<'a> {
	pub force: bool,
	pub project_directory: &'a Path,
	pub target: BuildTarget,
}

pub fn execute(options: &Options) -> Result<(), String> {
	let project = Project::load(options.project_directory, options.target)?;

	let mut event_listener = |event: BuildEvent| match event {
		BuildEvent::ExecutableBuilt(event) => match event.get_size() {
			Ok(size) => {
				println!("Executable compiled:");
				println!("  Path: {:?}", event.path);
				println!("  Size: {}", size);
			}
			Err(_err) => {
				println!("Unexpected error while getting size.");
			}
		},

		BuildEvent::LibraryBuilt(event) => {
			println!("Library compiled:");
			println!("  Path: {:?}", event.path);
		}

		_ => {}
	};

	let duration = build::build_duration(
		&BuildOptions {
			force: options.force,
			project: &project,
			target: options.target,
		},
		&mut event_listener,
	)?;

	println!("Build duration: {:?}.", duration);
	Ok(())
}
