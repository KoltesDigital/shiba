use crate::paths::BUILD_CACHE_DIRECTORY;
use crate::{Error, Result};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

pub fn compute_hash(value: &impl Hash) -> u64 {
	let mut hasher = DefaultHasher::new();
	value.hash(&mut hasher);
	hasher.finish()
}

pub fn get_build_cache_directory(value: &impl Hash) -> Result<PathBuf> {
	let hash = compute_hash(value);
	let path = BUILD_CACHE_DIRECTORY.join(format!("{:x}", hash));
	fs::create_dir_all(&path).map_err(|err| Error::failed_to_create_directory(&path, err))?;
	Ok(path)
}
