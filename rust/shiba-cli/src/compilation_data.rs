use std::collections::BTreeSet;
use std::path::PathBuf;

#[derive(Default, Hash)]
pub struct Common {
	pub link_dependencies: BTreeSet<PathBuf>,
	pub link_library_paths: BTreeSet<PathBuf>,
}

#[derive(Hash)]
pub enum CompilationJobKind {
	Asm,
	Cpp,
}

#[derive(Hash)]
pub struct CompilationJob {
	pub kind: CompilationJobKind,
	pub path: PathBuf,

	pub include_paths: BTreeSet<PathBuf>,
}

#[derive(Default, Hash)]
pub struct Compilation {
	pub jobs: Vec<CompilationJob>,
	pub include_paths: BTreeSet<PathBuf>,

	pub common: Common,
}

#[derive(Default, Hash)]
pub struct Linking {
	pub sources: Vec<PathBuf>,

	pub common: Common,
}
