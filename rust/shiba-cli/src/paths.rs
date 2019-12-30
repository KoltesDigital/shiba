use std::fs;
use std::path::PathBuf;

lazy_static! {
	pub static ref BUILD_CACHE_DIRECTORY: PathBuf = {
		let p = TEMP_DIRECTORY.join("build-cache");
		fs::create_dir_all(&p).unwrap();
		p
	};
	pub static ref BUILD_ROOT_DIRECTORY: PathBuf = {
		let p = TEMP_DIRECTORY.join("build");
		fs::create_dir_all(&p).unwrap();
		p
	};
	pub static ref DATA_DIRECTORY: PathBuf = {
		let p = dirs::data_dir().unwrap().join("shiba");
		fs::create_dir_all(&p).unwrap();
		p
	};
	pub static ref LOCAL_DATA_DIRECTORY: PathBuf = {
		let p = dirs::data_local_dir().unwrap().join("shiba");
		fs::create_dir_all(&p).unwrap();
		p
	};
	pub static ref TEMP_DIRECTORY: PathBuf = {
		let p = std::env::temp_dir().join("shiba");
		fs::create_dir_all(&p).unwrap();
		p
	};
	pub static ref USER_SETTINGS_DIRECTORY: PathBuf = {
		let p = dirs::home_dir().unwrap().join(".shiba");
		fs::create_dir_all(&p).unwrap();
		p
	};
}
