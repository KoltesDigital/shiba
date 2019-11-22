use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::{Cursor, Result};
use std::path::Path;

pub struct StoredHash<'a> {
	path: &'a Path,
	old_hash: Option<u64>,
	new_hash: Option<u64>,
}

impl<'a> StoredHash<'a> {
	pub fn new(path: &'a Path) -> Self {
		let old_hash = Self::load(path).ok();
		StoredHash {
			path,
			old_hash,
			new_hash: None,
		}
	}

	fn load(path: &Path) -> Result<u64> {
		let bytes = fs::read(path)?;
		let mut cursor = Cursor::new(bytes);
		let old_hash = cursor.read_u64::<BigEndian>()?;
		Ok(old_hash)
	}

	pub fn store(&self) -> Result<()> {
		if self.new_hash != self.old_hash {
			match self.new_hash {
				Some(hash) => {
					let mut bytes = vec![];
					bytes.write_u64::<BigEndian>(hash)?;
					fs::write(self.path, bytes)?;
				}
				None => fs::remove_file(self.path)?,
			}
		}
		Ok(())
	}

	pub fn get_updater<'b>(&'b mut self) -> Updater<'b, 'a>
	where
		'a: 'b,
	{
		Updater::new(self)
	}

	pub fn has_changed(&self) -> bool {
		self.new_hash != self.old_hash
	}
}

pub struct Updater<'b, 'a: 'b> {
	stored_hash: &'b mut StoredHash<'a>,
	hasher: DefaultHasher,
}

impl<'b, 'a: 'b> Updater<'b, 'a> {
	fn new(stored_hash: &'b mut StoredHash<'a>) -> Self {
		let hasher = DefaultHasher::new();
		Updater {
			stored_hash,
			hasher,
		}
	}

	pub fn add(&mut self, value: &impl Hash) {
		value.hash(&mut self.hasher)
	}
}

impl Drop for Updater<'_, '_> {
	fn drop(&mut self) {
		self.stored_hash.new_hash = Some(self.hasher.finish());
	}
}
