use serde::Deserialize;
use std::hash::Hash;

fn default_cl_args() -> Vec<String> {
	vec![
		"/O1",
		"/Oi",
		"/Oy",
		"/GR-",
		"/GS-",
		"/fp:fast",
		"/arch:IA32",
	]
	.into_iter()
	.map(|s| s.to_string())
	.collect()
}

#[derive(Debug, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub struct Cl {
	#[serde(default = "default_cl_args")]
	pub args: Vec<String>,
}

impl Default for Cl {
	fn default() -> Cl {
		Cl {
			args: default_cl_args(),
		}
	}
}

fn default_crinkler_args() -> Vec<String> {
	vec![
		"/ENTRY:main",
		"/PRIORITY:NORMAL",
		"/COMPMODE:FAST",
		"/RANGE:opengl32",
		"/REPORT:crinkler.html",
		// "/TRUNCATEFLOATS:16",
		"/UNSAFEIMPORT",
		"gdi32.lib",
		"opengl32.lib",
		"kernel32.lib",
		"user32.lib",
	]
	.into_iter()
	.map(|s| s.to_string())
	.collect()
}

#[derive(Debug, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub struct Crinkler {
	#[serde(default = "default_crinkler_args")]
	pub args: Vec<String>,
}

impl Default for Crinkler {
	fn default() -> Crinkler {
		Crinkler {
			args: default_crinkler_args(),
		}
	}
}

#[derive(Debug, Default, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub struct Settings {
	#[serde(default)]
	pub cl: Cl,
	#[serde(default)]
	pub crinkler: Crinkler,
}
