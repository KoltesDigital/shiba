use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

pub struct RunOptions<'a> {
	pub executable_path: &'a Path,
	pub project_directory: &'a Path,
}

pub fn run(options: &RunOptions) -> Result<(), String> {
	let mut process = Command::new(options.executable_path)
		.current_dir(options.project_directory)
		.spawn()
		.map_err(|err| err.to_string())?;

	let status = process.wait().map_err(|err| err.to_string())?;
	if !status.success() {
		return Err("Failed to run.".to_string());
	}

	Ok(())
}

pub fn run_duration(options: &RunOptions) -> Result<Duration, String> {
	let start = Instant::now();

	run(options)?;

	let duration = start.elapsed();
	Ok(duration)
}
