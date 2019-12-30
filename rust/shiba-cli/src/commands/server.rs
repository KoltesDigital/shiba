use crate::build::{self, BuildEvent, BuildOptions, BuildTarget};
use crate::types::{Pass, Variable};
use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{mpsc::channel, Arc, RwLock};
use std::thread::spawn;
use std::time::Duration;

pub struct Options<'a> {
	pub debounce_delay: Duration,
	pub ip: IpAddr,
	pub port: u16,
	pub project_directory: &'a Path,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "command")]
enum CommandKind {
	Build {
		force: Option<bool>,
		target: BuildTarget,
	},
	SetBuildOnChange {
		executable: bool,
		library: bool,
	},
	SetProjectDirectory {
		path: String,
	},
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Command {
	id: Option<String>,
	#[serde(flatten)]
	kind: CommandKind,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case", tag = "event")]
enum EventKind<'a> {
	BuildEnded {
		duration: Option<f32>,
		target: BuildTarget,
		successful: bool,
	},
	BuildStarted,
	ExecutableCompiled {
		path: &'a str,
		size: u64,
	},
	Error {
		message: &'a str,
	},
	LibraryCompiled {
		path: &'a str,
	},
	ShaderProvided {
		passes: &'a Vec<Pass>,
		target: BuildTarget,
		variables: &'a Vec<Variable>,
	},
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
struct Event<'a> {
	id: &'a Option<String>,
	#[serde(flatten)]
	kind: EventKind<'a>,
}

#[derive(Default)]
struct State {
	executable_build_on_change: bool,
	library_build_on_change: bool,
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

pub fn execute(options: &Options) -> Result<(), String> {
	let addr = SocketAddr::new(options.ip, options.port);
	let listener = TcpListener::bind(addr).map_err(|_| "Failed to start server.")?;
	println!("Listening on {}.", addr);

	let state = Arc::new(RwLock::new(State::default()));

	let (tx_watcher, rx_watcher) = channel();

	let (tx_command, rx_command) = channel::<Command>();
	let command_state = state.clone();
	let mut command_project_directory = options.project_directory.to_path_buf();
	spawn(move || {
		let mut watcher: RecommendedWatcher =
			Watcher::new(tx_watcher, Duration::from_secs_f32(0.3))
				.expect("Failed to create watcher.");
		watcher
			.watch(command_project_directory.clone(), RecursiveMode::Recursive)
			.expect("Failed to watch project directory");

		loop {
			match rx_command.recv() {
				Ok(command) => {
					let command_id = command.id.clone();
					match command.kind {
						CommandKind::Build { force, target } => {
							{
								let mut command_state = command_state.write().unwrap();
								command_state.broadcast(&Event {
									id: &command_id,
									kind: EventKind::BuildStarted,
								});
							}

							let event_listener = |event: BuildEvent| match event {
								BuildEvent::ExecutableCompiled(event) => match event.get_size() {
									Ok(size) => {
										let path = event.path.to_string_lossy();

										let mut command_state = command_state.write().unwrap();
										command_state.broadcast(&Event {
											id: &command_id,
											kind: EventKind::ExecutableCompiled {
												path: &path,
												size,
											},
										});
									}
									Err(err) => {
										let mut command_state = command_state.write().unwrap();
										command_state.broadcast(&Event {
											id: &command_id,
											kind: EventKind::Error { message: &err },
										});
									}
								},

								BuildEvent::LibraryCompiled(event) => {
									let path = event.path.to_string_lossy();

									let mut command_state = command_state.write().unwrap();
									command_state.broadcast(&Event {
										id: &command_id,
										kind: EventKind::LibraryCompiled { path: &path },
									});
								}

								BuildEvent::ShaderProvided(event) => {
									let mut command_state = command_state.write().unwrap();
									command_state.broadcast(&Event {
										id: &command_id,
										kind: EventKind::ShaderProvided {
											passes: &event.passes,
											target: event.target,
											variables: &event.variables,
										},
									});
								}
							};

							let result = build::build_duration(&BuildOptions {
								event_listener: &event_listener,
								force: force.unwrap_or(false),
								project_directory: &command_project_directory,
								target,
							});

							match result {
								Ok(duration) => {
									let mut command_state = command_state.write().unwrap();
									command_state.broadcast(&Event {
										id: &command.id,
										kind: EventKind::BuildEnded {
											duration: Some(duration.as_secs_f32()),
											target,
											successful: true,
										},
									});
								}

								Err(err) => {
									let mut command_state = command_state.write().unwrap();
									command_state.broadcast(&Event {
										id: &command.id,
										kind: EventKind::Error { message: &err },
									});
									command_state.broadcast(&Event {
										id: &command.id,
										kind: EventKind::BuildEnded {
											duration: None,
											target,
											successful: false,
										},
									});
								}
							};
						}

						CommandKind::SetBuildOnChange {
							executable,
							library,
						} => {
							let mut command_state = command_state.write().unwrap();

							command_state.executable_build_on_change = executable;
							command_state.library_build_on_change = library;
						}

						CommandKind::SetProjectDirectory { path } => {
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
					}
				}
				Err(err) => println!("Error while receiving command: {}", err),
			};
		}
	});

	let watcher_state = state.clone();
	let watcher_tx_command = tx_command.clone();
	spawn(move || loop {
		match rx_watcher.recv() {
			Ok(event) => match event {
				DebouncedEvent::Create(_)
				| DebouncedEvent::Remove(_)
				| DebouncedEvent::Rename(_, _)
				| DebouncedEvent::Rescan
				| DebouncedEvent::Write(_) => {
					let watcher_state = watcher_state.read().unwrap();

					if watcher_state.library_build_on_change {
						let _ = watcher_tx_command.send(Command {
							id: None,
							kind: CommandKind::Build {
								force: None,
								target: BuildTarget::Library,
							},
						});
					}

					if watcher_state.executable_build_on_change {
						let _ = watcher_tx_command.send(Command {
							id: None,
							kind: CommandKind::Build {
								force: None,
								target: BuildTarget::Executable,
							},
						});
					}
				}
				_ => {}
			},
			Err(err) => {
				panic!("Error while watching: {}", err);
			}
		};
	});

	let listening_state = state;
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
								Err(err) => println!("Failed to parse command: {}, {}", line, err),
							},
							Err(err) => println!("Error while reading line: {}", err),
						}
					}
				});
				listening_state.write().unwrap().streams.push(stream_clone);
			}
			Err(err) => println!("Error while listening: {}", err),
		}
	}

	Ok(())
}
