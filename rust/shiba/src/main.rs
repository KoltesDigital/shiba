#[macro_use]
extern crate lazy_static;

mod config_provider;
mod generators {
	pub mod blender_api;
}
mod parsers;
mod paths;
mod shader_providers {
	pub mod shiba;
}
mod subcommands {
	pub mod build;
	pub mod server;
}
mod template;
mod traits;
mod types;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about, author)]
struct Args {
	#[structopt(subcommand)]
	command: Option<Command>,
}

#[derive(Debug, StructOpt)]
enum Command {
	/// Builds the project (default)
	Build,
	/// Starts a server.
	Server,
}

fn main() -> Result<(), String> {
	let args = Args::from_args();

	if let Some(command) = &args.command {
		match command {
			Command::Build => subcommands::build::subcommand().map(|_| ()),
			Command::Server => subcommands::server::subcommand(),
		}
	} else {
		subcommands::build::subcommand().map(|_| ())
	}
}
