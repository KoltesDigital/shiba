pub mod msvc;
mod settings;

use crate::compiler::Compiler;
pub use settings::Settings;

pub trait LibraryCompiler: Compiler {}
