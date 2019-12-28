#[macro_use]
extern crate lazy_static;

macro_rules! template_enum {
	(
		$($variant:ident: $filename:expr),*,
	) => {
		#[allow(dead_code, non_camel_case_types)]
		enum Template {
			$($variant),*
		}

		impl Template {
			#[allow(dead_code)]
			fn as_array() -> Vec<(&'static str, &'static str)> {
				vec![
					$((stringify!($variant), include_str!(concat!("templates/", $filename, ".tera")))),*
				]
			}

			#[allow(dead_code)]
			fn name(&self) -> &'static str {
				match self {
					$(Template::$variant => stringify!($variant)),*
				}
			}
		}
	};
}

mod audio_synthesizers;
mod build;
mod code_map;
mod commands {
	pub mod build;
	pub mod server;
}
mod compiler;
mod configuration;
mod executable_compilers;
mod generator_utils {
	pub mod cpp;
	pub mod settings;
}
mod library_compilers;
mod parsers;
mod paths;
mod settings;
mod shader_codes;
mod shader_minifiers;
mod shader_providers;
mod types;

use crate::build::BuildTarget;
use std::net::IpAddr;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
enum Command {
	/// Builds the project as executable (default).
	BuildExecutable {
		#[structopt(short, long)]
		force: bool,
	},
	/// Builds the project as library.
	BuildLibrary {
		#[structopt(short, long)]
		force: bool,
	},
	/// Starts a server.
	Server {
		#[structopt(long, default_value = "127.0.0.1")]
		ip: IpAddr,
		#[structopt(short, long, default_value = "5184")]
		port: u16,
	},
}

impl Default for Command {
	fn default() -> Self {
		Command::BuildExecutable { force: false }
	}
}

#[derive(Debug, StructOpt)]
#[structopt(about, author)]
struct Args {
	#[structopt(subcommand)]
	command: Option<Command>,

	#[structopt(short, long, default_value = ".")]
	project_directory: PathBuf,
}

fn main() -> Result<(), String> {
	let args = Args::from_args();

	let command = args.command.unwrap_or_else(Command::default);
	match command {
		Command::BuildExecutable { force } => commands::build::execute(&commands::build::Options {
			force,
			project_directory: &args.project_directory,
			target: BuildTarget::Executable,
		})
		.map(|_| ()),
		Command::BuildLibrary { force } => commands::build::execute(&commands::build::Options {
			force,
			project_directory: &args.project_directory,
			target: BuildTarget::Library,
		})
		.map(|_| ()),
		Command::Server { ip, port } => commands::server::execute(&commands::server::Options {
			ip,
			port,
			project_directory: &args.project_directory,
		}),
	}
}
