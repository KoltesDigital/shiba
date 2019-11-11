use serde::Deserialize;
use serde_json;
use std::process::Command;
use std::str;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VSWhereItem {
	pub installation_path: String,
	pub is_complete: bool,
	pub is_launchable: bool,
}

pub enum Platform {
	X86,
	X64,
}

pub struct CommandGenerator {
	installation_path: String,
}

impl CommandGenerator {
	pub fn new() -> Result<Self, String> {
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

		Ok(CommandGenerator { installation_path })
	}

	pub fn command(&self, platform: Platform) -> Command {
		let platform = match platform {
			Platform::X64 => "x64",
			Platform::X86 => "x86",
		};

		let mut command = Command::new("cmd.exe");
		command
			.arg("/c")
			.arg("call")
			.arg(format!(
				r"{}\VC\Auxiliary\Build\vcvarsall.bat",
				self.installation_path,
			))
			.arg(platform)
			.arg("&&");

		command
	}
}
