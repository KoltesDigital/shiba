use std::convert::Into;
use std::error::Error as StdError;
use std::fmt;
use std::net::SocketAddr;
use std::path::PathBuf;
use tera;

#[derive(Debug)]
pub enum ErrorKind {
	ExecutionFailed(PathBuf),
	FailedToConvertUTF8(Vec<u8>),
	FailedToCopy(PathBuf, PathBuf),
	FailedToCreateDirectory(PathBuf),
	FailedToDeserialize(String),
	FailedToExecute(PathBuf),
	FailedToGetMetadata(PathBuf),
	FailedToListenTCP(SocketAddr),
	FailedToParse(String),
	FailedToRead(PathBuf),
	FailedToReadDirectory(PathBuf),
	FailedToRemoveDirectory(PathBuf),
	FailedToRenderTemplate(String),
	FailedToWrite(PathBuf),
	Message(String),
	PathHasInvalidFileName(PathBuf),
}

#[derive(Debug)]
pub struct Error {
	pub kind: ErrorKind,
	source: Option<Box<dyn StdError + Sync + Send>>,
}

impl Error {
	pub fn execution_failed(path: impl Into<PathBuf>) -> Self {
		Error {
			kind: ErrorKind::ExecutionFailed(path.into()),
			source: None,
		}
	}

	pub fn failed_to_convert_utf8(
		contents: &[u8],
		source: impl Into<Box<dyn StdError + Send + Sync>>,
	) -> Self {
		Error {
			kind: ErrorKind::FailedToConvertUTF8(contents.to_vec()),
			source: Some(source.into()),
		}
	}

	pub fn failed_to_copy(
		from: impl Into<PathBuf>,
		to: impl Into<PathBuf>,
		source: ::std::io::Error,
	) -> Self {
		Error {
			kind: ErrorKind::FailedToCopy(from.into(), to.into()),
			source: Some(source.into()),
		}
	}

	pub fn failed_to_create_directory(path: impl Into<PathBuf>, source: ::std::io::Error) -> Self {
		Error {
			kind: ErrorKind::FailedToCreateDirectory(path.into()),
			source: Some(source.into()),
		}
	}

	pub fn failed_to_deserialize(
		contents: &str,
		source: impl Into<Box<dyn StdError + Send + Sync>>,
	) -> Self {
		Error {
			kind: ErrorKind::FailedToDeserialize(contents.to_string()),
			source: Some(source.into()),
		}
	}

	pub fn failed_to_execute(path: impl Into<PathBuf>, source: ::std::io::Error) -> Self {
		Error {
			kind: ErrorKind::FailedToExecute(path.into()),
			source: Some(source.into()),
		}
	}

	pub fn failed_to_get_metadata(path: impl Into<PathBuf>, source: ::std::io::Error) -> Self {
		Error {
			kind: ErrorKind::FailedToGetMetadata(path.into()),
			source: Some(source.into()),
		}
	}

	pub fn failed_to_listen_tcp(addr: &SocketAddr, source: ::std::io::Error) -> Self {
		Error {
			kind: ErrorKind::FailedToListenTCP(*addr),
			source: Some(source.into()),
		}
	}

	pub fn failed_to_parse(contents: impl ToString) -> Self {
		Self {
			kind: ErrorKind::FailedToParse(contents.to_string()),
			source: None,
		}
	}

	pub fn failed_to_read(path: impl Into<PathBuf>, source: ::std::io::Error) -> Self {
		Error {
			kind: ErrorKind::FailedToRead(path.into()),
			source: Some(source.into()),
		}
	}

	pub fn failed_to_read_directory(path: impl Into<PathBuf>, source: ::std::io::Error) -> Self {
		Error {
			kind: ErrorKind::FailedToReadDirectory(path.into()),
			source: Some(source.into()),
		}
	}

	pub fn failed_to_remove_directory(path: impl Into<PathBuf>, source: ::std::io::Error) -> Self {
		Error {
			kind: ErrorKind::FailedToRemoveDirectory(path.into()),
			source: Some(source.into()),
		}
	}

	pub fn failed_to_render_template(name: &str, source: tera::Error) -> Self {
		Error {
			kind: ErrorKind::FailedToRenderTemplate(name.to_string()),
			source: Some(source.into()),
		}
	}

	pub fn failed_to_write(path: impl Into<PathBuf>, source: ::std::io::Error) -> Self {
		Error {
			kind: ErrorKind::FailedToWrite(path.into()),
			source: Some(source.into()),
		}
	}

	pub fn message(text: impl ToString) -> Self {
		Self {
			kind: ErrorKind::Message(text.to_string()),
			source: None,
		}
	}

	pub fn path_has_invalid_file_name(path: impl Into<PathBuf>) -> Self {
		Error {
			kind: ErrorKind::PathHasInvalidFileName(path.into()),
			source: None,
		}
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match &self.kind {
			ErrorKind::ExecutionFailed(path) => {
				write!(f, "Execution of '{}' failed.", path.to_string_lossy())
			}
			ErrorKind::FailedToConvertUTF8(data) => {
				write!(f, "Failed to convert UTF8 from '{:x?}'.", data)
			}
			ErrorKind::FailedToCopy(from, to) => write!(
				f,
				"Failed to copy from '{}' to '{}'.",
				from.to_string_lossy(),
				to.to_string_lossy(),
			),
			ErrorKind::FailedToCreateDirectory(path) => write!(
				f,
				"Failed to create directory '{}'.",
				path.to_string_lossy(),
			),
			ErrorKind::FailedToDeserialize(contents) => {
				write!(f, "Failed to deserialize '{}'.", contents)
			}
			ErrorKind::FailedToExecute(path) => {
				write!(f, "Failed to execute '{}'.", path.to_string_lossy())
			}
			ErrorKind::FailedToGetMetadata(path) => {
				write!(f, "Failed to get metadata '{}'.", path.to_string_lossy())
			}
			ErrorKind::FailedToListenTCP(addr) => write!(f, "Failed to listen to TCP '{}'.", addr),
			ErrorKind::FailedToParse(contents) => write!(f, "Failed to parse '{}'.", contents),
			ErrorKind::FailedToRead(path) => {
				write!(f, "Failed to read '{}'.", path.to_string_lossy())
			}
			ErrorKind::FailedToReadDirectory(path) => {
				write!(f, "Failed to read directory '{}'.", path.to_string_lossy())
			}
			ErrorKind::FailedToRemoveDirectory(path) => write!(
				f,
				"Failed to remove directory '{}'.",
				path.to_string_lossy(),
			),
			ErrorKind::FailedToRenderTemplate(name) => {
				write!(f, "Failed to render template '{}'.", name)
			}
			ErrorKind::FailedToWrite(path) => {
				write!(f, "Failed to write '{}'.", path.to_string_lossy())
			}
			ErrorKind::Message(message) => write!(f, "{}", message),
			ErrorKind::PathHasInvalidFileName(path) => {
				write!(f, "Path '{}' has invalid filename.", path.to_string_lossy())
			}
		}
	}
}

impl StdError for Error {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		self.source
			.as_ref()
			.map(|c| &**c as &(dyn StdError + 'static))
	}
}

impl From<&str> for Error {
	fn from(text: &str) -> Self {
		Self::message(text)
	}
}

impl From<String> for Error {
	fn from(text: String) -> Self {
		Self::message(text)
	}
}

pub type Result<T> = ::std::result::Result<T, Error>;
