use crate::paths::{BUILD_CACHE_DIRECTORY, BUILD_ROOT_DIRECTORY};
use crate::{Error, Result};
use std::fs;

pub fn execute() -> Result<()> {
	fs::remove_dir_all(&*BUILD_CACHE_DIRECTORY)
		.map_err(|err| Error::failed_to_remove_directory(&*BUILD_CACHE_DIRECTORY, err))?;
	fs::remove_dir_all(&*BUILD_ROOT_DIRECTORY)
		.map_err(|err| Error::failed_to_remove_directory(&*BUILD_ROOT_DIRECTORY, err))?;
	Ok(())
}
