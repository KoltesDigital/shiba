#[macro_use]
extern crate lazy_static;

mod configuration;
mod custom_codes;
mod generators {
	pub mod blender_api;
}
mod parsers;
mod paths;
mod settings;
mod shader_providers {
	pub mod shiba;
}
mod shader_codes;
mod stored_hash;
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
	Build {
		#[structopt(short, long)]
		force: bool,
	},
	/// Starts a server.
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
		Command::Build { force } => subcommands::build::subcommand(&subcommands::build::Options {
			diff: false,
			force,
			project_directory: &args.project_directory,
		})
		.map(|_| ()),
		Command::Server => subcommands::server::subcommand(&args.project_directory),
	}
}
