#[macro_use]
extern crate lazy_static;

mod configuration;
mod custom_codes;
mod generators {
	pub mod blender_api;
}
mod parsers;
mod paths;
mod shader_providers {
	pub mod shiba;
}
mod settings;
mod subcommands {
	pub mod build;
	pub mod server;
}
mod template;
mod traits;
mod types;

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
enum Command {
	/// Builds the project (default)
	Build,
	/// Starts a server.
	Server,
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

	if let Some(command) = &args.command {
		match command {
			Command::Build => subcommands::build::subcommand(&args.project_directory).map(|_| ()),
			Command::Server => subcommands::server::subcommand(&args.project_directory),
		}
	} else {
		subcommands::build::subcommand(&args.project_directory).map(|_| ())
	}
}
