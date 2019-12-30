use crate::paths::{BUILD_CACHE_DIRECTORY, BUILD_ROOT_DIRECTORY};
use std::fs;

pub fn execute() -> Result<(), String> {
	fs::remove_dir_all(&*BUILD_CACHE_DIRECTORY).map_err(|err| err.to_string())?;
	fs::remove_dir_all(&*BUILD_ROOT_DIRECTORY).map_err(|err| err.to_string())?;
	Ok(())
}
