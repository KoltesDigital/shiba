use serde::Serialize;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use tera::Tera;

pub type CodeMap = BTreeMap<String, String>;

#[derive(Serialize)]
struct Context {
	development: bool,
}

pub fn load_project_codes(project_directory: &Path, development: bool) -> Result<CodeMap, String> {
	let context = Context { development };

	let codes = fs::read_dir(&project_directory)
		.map_err(|_| "Failed to read directory.")?
		.filter_map(|entry| {
			let entry = entry.unwrap();
			if entry.file_type().unwrap().is_file() {
				let path = entry.path();
				if path.extension() == Some(OsStr::new("cpp")) {
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
				} else {
					None
				}
			} else {
				None
			}
		})
		.collect();

	Ok(codes)
}
