use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Platform {
	X64,
	X86,
}

pub trait CompilationJobEmitter {
	fn requires_asm_compiler(&self) -> bool;
	fn requires_cpp_compiler(&self) -> bool;
}

pub trait PlatformDependent {
	fn get_possible_platforms(&self) -> &'static BTreeSet<Platform>;
}
