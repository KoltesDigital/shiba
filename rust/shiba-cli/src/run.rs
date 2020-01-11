use crate::{Error, Result};
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

pub struct RunOptions<'a> {
	pub executable_path: &'a Path,
	pub project_directory: &'a Path,
}

pub fn run(options: &RunOptions) -> Result<()> {
	let mut process = Command::new(options.executable_path)
		.current_dir(options.project_directory)
		.spawn()
		.map_err(|err| Error::failed_to_execute(options.executable_path, err))?;

	let status = process.wait().unwrap();
	if !status.success() {
		return Err(Error::execution_failed(options.executable_path));
	}

	Ok(())
}

pub fn run_duration(options: &RunOptions) -> Result<Duration> {
	let start = Instant::now();

	run(options)?;

	let duration = start.elapsed();
	Ok(duration)
}
