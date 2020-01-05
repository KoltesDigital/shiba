pub mod crinkler;
pub mod msvc;

use crate::build::BuildOptions;
use crate::compilation::{Platform, PlatformDependent};
use crate::compilation_data::Linking;
use std::path::PathBuf;

#[derive(Hash)]
pub struct LinkOptions<'a> {
	pub linking: &'a Linking,
	pub platform: Platform,
}

pub trait Linker: PlatformDependent {
	fn link(&self, build_options: &BuildOptions, options: &LinkOptions) -> Result<PathBuf, String>;
}
