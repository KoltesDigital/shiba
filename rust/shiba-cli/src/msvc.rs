use crate::compilation::Platform;
use encoding::all::UTF_8;
use encoding::{DecoderTrap, Encoding};
use serde::Deserialize;
use serde_json;
use std::process::Command;
use std::str;

fn default_true() -> bool {
	true
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VSWhereItem {
	pub installation_path: String,
	#[serde(default = "default_true")]
	pub is_complete: bool,
	#[serde(default = "default_true")]
	pub is_launchable: bool,
}

#[derive(Hash)]
pub struct CommandGeneratorInputs<'a> {
	pub installation_path: &'a String,
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
		let json = UTF_8
			.decode(&vswhere.stdout, DecoderTrap::Ignore)
			.map_err(|_| "Failed to convert UTF8.")?;
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

	pub fn get_inputs(&self) -> CommandGeneratorInputs {
		CommandGeneratorInputs {
			installation_path: &self.installation_path,
		}
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
