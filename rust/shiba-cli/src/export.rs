use crate::paths::TEMP_DIRECTORY;
use crate::project_data::Project;
use crate::{Error, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ExportOutput {
	Directory,
	#[serde(rename = "7z")]
	SevenZ,
	Zip,
}

impl FromStr for ExportOutput {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self> {
		match s {
			"directory" => Ok(ExportOutput::Directory),
			"7z" => Ok(ExportOutput::SevenZ),
			"zip" => Ok(ExportOutput::Zip),
			_ => Err(Error::message("Invalid output variant.")),
		}
	}
}

pub struct ExportOptions<'a> {
	pub build_path: &'a Path,
	pub directory: &'a Path,
	pub output: ExportOutput,
	pub project: &'a Project,
	pub static_files: &'a [PathBuf],
}

pub fn export(options: &ExportOptions) -> Result<PathBuf> {
	let mut export_directory = PathBuf::from(options.directory);
	if export_directory.is_relative() {
		export_directory = options.project.directory.join(export_directory);
	}

	// Directly use the final path if exporting as directory.
	let temp_directory = if options.output == ExportOutput::Directory {
		export_directory.clone()
	} else {
		TEMP_DIRECTORY.join("export")
	};
	let temp_named_directory = temp_directory.join(&options.project.settings.name);

	if temp_named_directory.exists() {
		fs::remove_dir_all(&temp_named_directory)
			.map_err(|err| Error::failed_to_remove_directory(&temp_named_directory, err))?;
	}

	fs::create_dir_all(&temp_named_directory)
		.map_err(|err| Error::failed_to_create_directory(&temp_named_directory, err))?;

	let mut target_filename = options.project.settings.name.clone();
	if let Some(extension) = options.build_path.extension() {
		target_filename.push('.');
		target_filename.push_str(&extension.to_string_lossy());
	}

	let copy_to = temp_named_directory.join(target_filename);
	fs::copy(&options.build_path, &copy_to)
		.map_err(|err| Error::failed_to_copy(&options.build_path, &copy_to, err))?;

	for static_file in options.static_files {
		if let Some(file_name) = static_file.file_name() {
			let copy_to = temp_named_directory.join(&file_name);
			fs::copy(&static_file, &copy_to)
				.map_err(|err| Error::failed_to_copy(&static_file, &copy_to, err))?;
		}
	}

	let output_path = match options.output {
		ExportOutput::Directory => {
			// Already done.
			temp_named_directory
		}

		output => {
			if export_directory.exists() {
				fs::remove_dir_all(&export_directory)
					.map_err(|err| Error::failed_to_remove_directory(&export_directory, err))?;
			}

			fs::create_dir_all(&export_directory)
				.map_err(|err| Error::failed_to_create_directory(&export_directory, err))?;

			let sevenz_path = &options.project.configuration.get_path("7z");

			match output {
				ExportOutput::SevenZ => {
					let output_path =
						export_directory.join(format!("{}.7z", options.project.settings.name));

					let mut archiving = Command::new(&sevenz_path)
						.arg("a")
						.arg("-t7z")
						.arg(&output_path)
						.arg(&options.project.settings.name)
						.current_dir(&temp_directory)
						.spawn()
						.map_err(|err| Error::failed_to_execute(&sevenz_path, err))?;

					let status = archiving.wait().unwrap();
					if !status.success() {
						return Err(Error::execution_failed(&sevenz_path));
					}

					output_path
				}

				ExportOutput::Zip => {
					let output_path =
						export_directory.join(format!("{}.zip", options.project.settings.name));

					let mut archiving = Command::new(&sevenz_path)
						.arg("a")
						.arg("-tzip")
						.arg(&output_path)
						.arg(&options.project.settings.name)
						.current_dir(&temp_directory)
						.spawn()
						.map_err(|err| Error::failed_to_execute(&sevenz_path, err))?;

					let status = archiving.wait().unwrap();
					if !status.success() {
						return Err(Error::execution_failed(&sevenz_path));
					}

					output_path
				}

				ExportOutput::Directory => unreachable!(),
			}
		}
	};

	Ok(output_path)
}
