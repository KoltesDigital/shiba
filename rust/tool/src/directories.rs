use std::fs;
use std::path::PathBuf;

lazy_static! {
	pub static ref TMP: PathBuf = {
		let p = std::env::temp_dir().join("shiba");
		fs::create_dir_all(&p).unwrap();
		p
	};
	pub static ref USER_SETTINGS: PathBuf = {
		let p = dirs::home_dir().unwrap().join(".shiba");
		fs::create_dir_all(&p).unwrap();
		p
	};
}
