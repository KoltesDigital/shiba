pub mod crinkler;
pub mod msvc;
mod settings;

use crate::compiler::Compiler;
pub use settings::Settings;

pub trait ExecutableCompiler: Compiler {}
