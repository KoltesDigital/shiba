use crate::subcommands;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener};
use std::str::FromStr;
use std::thread::spawn;

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case", tag = "command")]
enum Command {
	Build,
}

#[derive(Serialize)]
struct BlenderApiAvailableEvent<'a> {
	pub path: &'a String,
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

pub fn subcommand() -> Result<(), String> {
	let addr = SocketAddr::from_str("127.0.0.1:5184").map_err(|_| "Invalid socket address.")?;
	let listener = TcpListener::bind(addr).map_err(|_| "Failed to start server.")?;

	for stream in listener.incoming() {
		match stream {
			Ok(mut stream) => {
				spawn(move || {
					if let Ok(stream_clone) = stream.try_clone() {
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
										Command::Build => match subcommands::build::subcommand() {
											Ok(path) => send(&Event::BlenderApiAvailable(
												BlenderApiAvailableEvent { path: &path },
											)),
											Err(err) => send(&Event::Error(ErrorEvent {
												message: &err.to_string(),
											})),
										},
									},
									Err(_) => send(&Event::InvalidCommand),
								},
								Err(_) => println!("Error while reading line."),
							}
						}
					} else {
						println!("Failed to clone stream.");
					}
				});
			}
			Err(_) => println!("Error while listening."),
		}
	}

	Ok(())
}
