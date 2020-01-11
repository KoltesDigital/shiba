use crate::build::BuildTarget;
use crate::{Error, Result};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};

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
	pub fn load(project_directory: &Path, options: &LoadOptions) -> Result<Self> {
		let mut code_files = vec![];
		let mut static_files = vec![];

		let entries = fs::read_dir(&project_directory)
			.map_err(|err| Error::failed_to_read_directory(&project_directory, err))?;

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

	pub fn get_compiler_codes(&self, development: bool, target: BuildTarget) -> Result<CodeMap> {
		#[derive(Serialize)]
		struct OwnContext {
			development: bool,
			target: BuildTarget,
		}

		let context = OwnContext {
			development,
			target,
		};

		self.code_files
			.iter()
			.map(|path| {
				let name = path
					.file_stem()
					.unwrap()
					.to_str()
					.expect("Failed to convert path.")
					.to_string();

				let contents =
					fs::read_to_string(&path).map_err(|err| Error::failed_to_read(&path, err))?;

				let mut tera = Tera::default();

				tera.add_raw_template(&name, &contents)
					.expect("Failed to add template.");

				let contents = tera
					.render(
						&name,
						&Context::from_serialize(&context).expect("Failed to create context."),
					)
					.map_err(|err| Error::failed_to_render_template(&name, err))?;

				Ok((name, contents))
			})
			.collect()
	}

	pub fn get_static_files(&self) -> &Vec<PathBuf> {
		&self.static_files
	}
}
