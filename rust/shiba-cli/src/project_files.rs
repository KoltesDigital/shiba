use crate::build::BuildTarget;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use tera::Tera;

pub type IsPathHandled<'a> = Box<dyn Fn(&Path) -> bool + 'a>;

pub trait FileConsumer {
	fn get_is_path_handled<'b, 'a: 'b>(&'a self) -> IsPathHandled<'b>;
}

pub type CodeMap = BTreeMap<String, String>;

#[derive(Debug)]
pub struct ProjectFiles {
	code_files: Vec<PathBuf>,
	static_files: Vec<PathBuf>,
}

pub struct LoadOptions<'a> {
	pub compiler_paths: &'a [IsPathHandled<'a>],
	pub ignore_paths: &'a [IsPathHandled<'a>],
}

impl ProjectFiles {
	pub fn load(project_directory: &Path, options: &LoadOptions) -> Result<Self, String> {
		let mut code_files = vec![];
		let mut static_files = vec![];

		let entries = fs::read_dir(&project_directory).map_err(|_| "Failed to read directory.")?;

		for entry in entries {
			let entry = entry.unwrap();
			if entry.file_type().unwrap().is_file() {
				let path = entry.path();

				if options.ignore_paths.iter().any(|handler| handler(&path)) {
					// Skip.
				} else if options.compiler_paths.iter().any(|handler| handler(&path)) {
					code_files.push(path);
				} else {
					static_files.push(path);
				}
			}
		}

		Ok(ProjectFiles {
			code_files,
			static_files,
		})
	}

	pub fn get_compiler_codes(
		&self,
		development: bool,
		target: BuildTarget,
	) -> Result<CodeMap, String> {
		#[derive(Serialize)]
		struct Context {
			development: bool,
			target: BuildTarget,
		}

		let context = Context {
			development,
			target,
		};

		let codes = self
			.code_files
			.iter()
			.filter_map(|path| {
				let name = path
					.file_stem()
					.unwrap()
					.to_str()
					.expect("Failed to convert path.")
					.to_string();

				let contents = fs::read_to_string(&path).expect("Failed to read file.");

				let mut tera = Tera::default();

				match tera.add_raw_template(&name, &contents) {
					Ok(()) => match tera.render(&name, &context) {
						Ok(contents) => Some((name, contents)),
						Err(err) => {
							println!("{}", err);
							None
						}
					},
					Err(err) => {
						println!("{}", err);
						None
					}
				}
			})
			.collect();

		Ok(codes)
	}

	pub fn get_static_files(&self) -> &Vec<PathBuf> {
		&self.static_files
	}
}
