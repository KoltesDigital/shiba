use crate::generators;
use crate::subcommands;
use crate::types::Pass;
use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{mpsc::channel, Arc, RwLock};
use std::thread::spawn;
use std::time::Duration;

fn default_command_build_blender_api_diff() -> bool {
	false
}

fn default_command_build_blender_api_force() -> bool {
	false
}

fn default_command_build_executable_force() -> bool {
	false
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "command")]
enum Command {
	BuildBlenderApi {
		#[serde(default = "default_command_build_blender_api_diff")]
		diff: bool,
		#[serde(default = "default_command_build_blender_api_force")]
		force: bool,
	},
	BuildExecutable {
		#[serde(default = "default_command_build_executable_force")]
		force: bool,
	},
	GetBlenderApiPath,
	SetProjectDirectory {
		path: String,
	},
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case", tag = "kind")]
enum BuildStartedKind {
	BlenderApi,
	Executable,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case", tag = "result")]
enum BuildEndedResult<'a> {
	BlenderApi,
	Executable,
	Nothing,
	ShaderPasses { passes: &'a [Pass] },
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case", tag = "event")]
enum Event<'a> {
	BuildStarted {
		#[serde(flatten)]
		kind: BuildStartedKind,
	},
	BuildEnded {
		#[serde(flatten)]
		result: BuildEndedResult<'a>,
	},
	BlenderApiPath {
		path: &'a Path,
	},
	Error {
		message: &'a str,
	},
}

#[derive(Default)]
struct State {
	streams: Vec<TcpStream>,
}

impl State {
	pub fn broadcast(&mut self, event: &Event) {
		let json = serde_json::to_string(event).unwrap();
		for stream in &mut self.streams {
			stream
				.write_all(json.as_bytes())
				.expect("Failed to write to stream.");
			stream.write_all(b"\n").expect("Failed to write to stream.");
		}
	}
}

pub fn subcommand(project_directory: &Path) -> Result<(), String> {
	let addr = SocketAddr::from_str("127.0.0.1:5184").map_err(|_| "Invalid socket address.")?;
	let listener = TcpListener::bind(addr).map_err(|_| "Failed to start server.")?;

	let state = Arc::new(RwLock::new(State::default()));

	let (tx_watcher, rx_watcher) = channel();

	let (tx_command, rx_command) = channel();
	let command_state = state.clone();
	let mut command_project_directory = project_directory.to_path_buf();
	spawn(move || {
		let mut watcher: RecommendedWatcher =
			Watcher::new(tx_watcher, Duration::from_secs_f32(0.1))
				.expect("Failed to create watcher.");
		watcher
			.watch(command_project_directory.clone(), RecursiveMode::Recursive)
			.expect("Failed to watch project directory");
		loop {
			match rx_command.recv() {
				Ok(command) => match command {
					Command::BuildBlenderApi { diff, force } => {
						{
							let mut state = command_state.write().unwrap();
							state.broadcast(&Event::BuildStarted {
								kind: BuildStartedKind::BlenderApi,
							});
						}

						match subcommands::build_blender_api::subcommand(
							&subcommands::build_blender_api::Options {
								diff,
								force,
								project_directory: &command_project_directory,
							},
						) {
							Ok(result) => {
								let result = match &result {
									subcommands::build_blender_api::ResultKind::BlenderAPIAvailable => BuildEndedResult::BlenderApi,
									subcommands::build_blender_api::ResultKind::Nothing => BuildEndedResult::Nothing,
									subcommands::build_blender_api::ResultKind::ShaderPassesAvailable(passes) => BuildEndedResult::ShaderPasses{ passes: &passes },
								};

								let mut state = command_state.write().unwrap();
								state.broadcast(&Event::BuildEnded { result });
							}
							Err(err) => {
								let mut state = command_state.write().unwrap();
								state.broadcast(&Event::Error {
									message: &err.to_string(),
								});
							}
						}
					}

					Command::BuildExecutable { force } => {
						{
							let mut state = command_state.write().unwrap();
							state.broadcast(&Event::BuildStarted {
								kind: BuildStartedKind::Executable,
							});
						}

						match subcommands::build_executable::subcommand(
							&subcommands::build_executable::Options {
								force,
								project_directory: &command_project_directory,
							},
						) {
							Ok(result) => {
								let result = match &result {
									subcommands::build_executable::ResultKind::ExecutableAvailable => BuildEndedResult::Executable,
									subcommands::build_executable::ResultKind::Nothing => BuildEndedResult::Nothing,
								};

								let mut state = command_state.write().unwrap();
								state.broadcast(&Event::BuildEnded { result });
							}
							Err(err) => {
								let mut state = command_state.write().unwrap();
								state.broadcast(&Event::Error {
									message: &err.to_string(),
								});
							}
						}
					}

					Command::GetBlenderApiPath => {
						let path = generators::blender_api::Generator::get_path();
						let mut state = command_state.write().unwrap();
						state.broadcast(&Event::BlenderApiPath { path: &path })
					}

					Command::SetProjectDirectory { path } => {
						if let Err(err) = watcher.unwatch(&command_project_directory) {
							println!("Failed to unwatch project directory: {}", err);
						}
						command_project_directory = PathBuf::from(path);
						if let Err(err) =
							watcher.watch(&command_project_directory, RecursiveMode::Recursive)
						{
							println!("Failed to watch project directory: {}", err);
						}
					}
				},
				Err(err) => println!("Error while receiving command: {}", err),
			};
		}
	});

	let watcher_tx_command = tx_command.clone();
	spawn(move || loop {
		match rx_watcher.recv() {
			Ok(event) => match event {
				DebouncedEvent::Create(_)
				| DebouncedEvent::Remove(_)
				| DebouncedEvent::Rename(_, _)
				| DebouncedEvent::Rescan
				| DebouncedEvent::Write(_) => {
					let _ = watcher_tx_command.send(Command::BuildBlenderApi {
						diff: true,
						force: false,
					});
				}
				_ => {}
			},
			Err(err) => {
				panic!("Error while watching: {}", err);
			}
		};
	});

	for stream in listener.incoming() {
		match stream {
			Ok(stream) => {
				let tx_command = tx_command.clone();
				let stream_clone = stream.try_clone().expect("Failed to clone stream.");
				spawn(move || {
					let stream = BufReader::new(stream);
					for line in stream.lines() {
						match &line {
							Ok(line) => match serde_json::from_str::<Command>(line.as_str()) {
								Ok(command) => {
									let _ = tx_command.send(command);
								}
								Err(_) => println!("Failed to parse command: {}", line),
							},
							Err(err) => println!("Error while reading line: {}", err),
						}
					}
				});
				state.write().unwrap().streams.push(stream_clone);
			}
			Err(err) => println!("Error while listening: {}", err),
		}
	}

	Ok(())
}
