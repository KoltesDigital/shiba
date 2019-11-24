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

mod audio_synthesizers {
	pub mod none;
	pub mod oidos;
}
mod code_map;
mod commands {
	pub mod build_blender_api;
	pub mod build_executable;
	pub mod server;
}
mod configuration;
mod generators {
	pub mod blender_api;
	pub mod crinkler;
	pub mod executable;
}
mod generator_utils {
	pub mod cpp;
	pub mod settings;
}
mod parsers;
mod paths;
mod settings;
mod shader_minifiers {
	pub mod shader_minifier;
}
mod shader_providers {
	pub mod shiba;
}
mod shader_codes;
mod stored_hash;
mod traits;
mod types;

use std::net::IpAddr;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
enum Command {
	/// Builds the project (default)
	Build {
		#[structopt(short, long)]
		force: bool,
	},
	/// Builds the Blender API
	BuildBlender {
		#[structopt(short, long)]
		force: bool,
	},
	/// Starts a server
	Server {
		#[structopt(long, default_value = "127.0.0.1")]
		ip: IpAddr,
		#[structopt(short, long, default_value = "5184")]
		port: u16,
	},
}

impl Default for Command {
	fn default() -> Self {
		Command::Build { force: false }
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
		Command::Build { force } => {
			commands::build_executable::subcommand(&commands::build_executable::Options {
				force,
				project_directory: &args.project_directory,
			})
			.map(|_| ())
		}
		Command::BuildBlender { force } => {
			commands::build_blender_api::subcommand(&commands::build_blender_api::Options {
				diff: false,
				force,
				project_directory: &args.project_directory,
			})
			.map(|_| ())
		}
		Command::Server { ip, port } => commands::server::subcommand(&commands::server::Options {
			ip,
			port,
			project_directory: &args.project_directory,
		}),
	}
}
