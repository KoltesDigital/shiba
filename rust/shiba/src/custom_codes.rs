use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

pub type CustomCodes = BTreeMap<String, String>;

pub fn load(project_directory: &Path) -> Result<CustomCodes, String> {
	let map = fs::read_dir(&project_directory)
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
					Some((name, contents))
				} else {
					None
				}
			} else {
				None
			}
		})
		.collect();
	Ok(map)
}
