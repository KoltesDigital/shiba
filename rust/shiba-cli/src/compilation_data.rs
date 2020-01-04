use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Deserialize, Hash, Serialize)]
pub struct ClCompilation {
	pub args: Vec<String>,
}

#[derive(Clone, Default, Deserialize, Hash, Serialize)]
pub struct CrinklerCompilation {
	pub args: Vec<String>,
}

#[derive(Clone, Default, Deserialize, Hash, Serialize)]
pub struct LinkCompilation {
	pub args: Vec<String>,
}

#[derive(Clone, Default, Deserialize, Hash, Serialize)]
pub struct Compilation {
	pub cl: ClCompilation,
	pub crinkler: CrinklerCompilation,
	pub link: LinkCompilation,
}
