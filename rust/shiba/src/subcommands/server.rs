use crate::subcommands;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::thread::spawn;

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case", tag = "command")]
enum Command {
	Build,
	SetProjectDirectory { path: String },
}

#[derive(Serialize)]
struct BlenderApiAvailableEvent<'a> {
	pub path: &'a Path,
}

#[derive(Serialize)]
struct ErrorEvent<'a> {
	pub message: &'a String,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case", tag = "event")]
enum Event<'a> {
	BlenderApiAvailable(BlenderApiAvailableEvent<'a>),
	Error(ErrorEvent<'a>),
	InvalidCommand,
}

pub fn subcommand(project_directory: &Path) -> Result<(), String> {
	let addr = SocketAddr::from_str("127.0.0.1:5184").map_err(|_| "Invalid socket address.")?;
	let listener = TcpListener::bind(addr).map_err(|_| "Failed to start server.")?;
	let clients = Arc::new(RwLock::new(Vec::new()));

	for stream in listener.incoming() {
		match stream {
			Ok(stream) => {
				let mut project_directory = project_directory.to_path_buf();
				if let Ok(stream_clone) = stream.try_clone() {
					let thread = spawn(move || {
						let mut stream = stream_clone.try_clone().expect("Failed to clone stream.");
						let mut send = |event: &Event| {
							let json = serde_json::to_string(event).unwrap();
							stream
								.write_all(json.as_bytes())
								.expect("Failed to write to stream.");
							stream.write_all(b"\n").expect("Failed to write to stream.");
						};

						let stream = BufReader::new(stream_clone);
						for line in stream.lines() {
							match &line {
								Ok(line) => match serde_json::from_str::<Command>(line.as_str()) {
									Ok(command) => match command {
										Command::Build => {
											match subcommands::build::subcommand(&project_directory)
											{
												Ok(path) => send(&Event::BlenderApiAvailable(
													BlenderApiAvailableEvent { path: &path },
												)),
												Err(err) => send(&Event::Error(ErrorEvent {
													message: &err.to_string(),
												})),
											}
										}
										Command::SetProjectDirectory { path } => {
											project_directory = PathBuf::from(path);
										}
									},
									Err(_) => send(&Event::InvalidCommand),
								},
								Err(err) => println!("Error while reading line: {}", err),
							}
						}
					});
					let stream_clone = stream.try_clone().expect("Failed to clone stream.");
					clients.write().unwrap().push((thread, stream_clone));
				} else {
					println!("Failed to clone stream.");
				}
			}
			Err(err) => println!("Error while listening: {}", err),
		}
	}

	for (thread, _) in clients.write().unwrap().drain(..) {
		thread.join().unwrap();
	}

	Ok(())
}
