#[macro_use]
extern crate lazy_static;

mod configuration;
mod custom_codes;
mod generators {
	pub mod blender_api;
	pub mod crinkler;
	pub mod executable;
}
mod generator_utils {
	pub mod cpp;
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
mod subcommands {
	pub mod build_blender_api;
	pub mod build_executable;
	pub mod server;
}
mod traits;
mod types;

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
	Server,
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
			subcommands::build_executable::subcommand(&subcommands::build_executable::Options {
				force,
				project_directory: &args.project_directory,
			})
			.map(|_| ())
		}
		Command::BuildBlender { force } => {
			subcommands::build_blender_api::subcommand(&subcommands::build_blender_api::Options {
				diff: false,
				force,
				project_directory: &args.project_directory,
			})
			.map(|_| ())
		}
		Command::Server => subcommands::server::subcommand(&args.project_directory),
	}
}
