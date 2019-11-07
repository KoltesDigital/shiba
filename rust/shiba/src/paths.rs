use serde::Deserialize;
use serde_json;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::str;

lazy_static! {
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VSWhereItem {
	pub installation_path: String,
	pub is_complete: bool,
	pub is_launchable: bool,
}

pub enum MSVCPlatform {
	X86,
	X64,
}

pub fn msvc(platform: MSVCPlatform) -> Result<PathBuf, String> {
	let vswhere =
		Command::new(r"C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe")
			.arg("-format")
			.arg("json")
			.output()
			.map_err(|_| "Failed to execute vswhere.")?;
	let json = str::from_utf8(&vswhere.stdout).map_err(|_| "Failed to convert UTF8.")?;
	let items: Vec<VSWhereItem> =
		serde_json::from_str(&json).map_err(|_| "Failed to parse JSON.")?;
	let installation_path = items
		.iter()
		.find(|&item| item.is_complete && item.is_launchable)
		.ok_or_else(|| "Cannot find any VS installation.".to_string())?
		.installation_path
		.clone();
	Ok(PathBuf::from(installation_path))
	/*
	let msvc_root = PathBuf::from(installation_path)
		.join("VC")
		.join("Tools")
		.join("MSVC");
	let mut entries = fs::read_dir(msvc_root).map_err(|_| "Failed to read MSVC directory.")?;
	let msvc_dir = entries
		.nth(0)
		.ok_or_else(|| "Cannot find any MSVC directory.".to_string())?
		.map_err(|_| "Cannot read MSVC directory.".to_string())?
		.path()
		.join("bin");
	let msvc_dir = match platform {
		MSVCPlatform::X86 => msvc_dir.join("Hostx86").join("x86"),
		MSVCPlatform::X64 => msvc_dir.join("Hostx64").join("x64"),
	};
	Ok(msvc_dir)*/
}
