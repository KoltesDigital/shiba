use crate::generators;
use crate::subcommands;
use crate::types::Pass;
use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{mpsc::channel, Arc, Mutex, RwLock};
use std::thread::spawn;
use std::time::Duration;

pub struct Options<'a> {
	pub ip: IpAddr,
	pub port: u16,
	pub project_directory: &'a Path,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "command")]
struct SetBuildExecutableCommand {
	build_executable: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "command")]
enum Command {
	Build {
		#[serde(default)]
		diff: bool,
		#[serde(default)]
		force: bool,
	},
	GetBlenderApiPath,
	GetExecutableSize,
	SetBuildExecutable(SetBuildExecutableCommand),
	SetProjectDirectory {
		path: String,
	},
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case", tag = "kind")]
enum BuildStartedWhat {
	BlenderApi,
	Executable,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case", tag = "kind")]
enum BuildEndedWhat {
	BlenderApi,
	Executable,
	ShaderPasses { passes: Vec<Pass> },
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case", tag = "event")]
enum Event<'a> {
	BuildStarted { what: &'a [BuildStartedWhat] },
	BuildEnded { what: &'a [BuildEndedWhat] },
	BlenderApiPath { path: &'a Path },
	Error { message: &'a str },
	ExecutableSize { size: u64 },
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

pub fn subcommand(options: &Options) -> Result<(), String> {
	let addr = SocketAddr::new(options.ip, options.port);
	let listener = TcpListener::bind(addr).map_err(|_| "Failed to start server.")?;
	println!("Listening on {}.", addr);

	let state = Arc::new(RwLock::new(State::default()));

	let (tx_watcher, rx_watcher) = channel();

	let (tx_command, rx_command) = channel();
	let command_state = state.clone();
	let mut command_project_directory = options.project_directory.to_path_buf();
	spawn(move || {
		let command_build_blender_api = true;
		let mut command_build_executable = false;

		let mut watcher: RecommendedWatcher =
			Watcher::new(tx_watcher, Duration::from_secs_f32(0.1))
				.expect("Failed to create watcher.");
		watcher
			.watch(command_project_directory.clone(), RecursiveMode::Recursive)
			.expect("Failed to watch project directory");
		loop {
			match rx_command.recv() {
				Ok(command) => match command {
					Command::Build { diff, force } => {
						let mut threads = vec![];
						let mut started_what = vec![];
						let ended_what = Arc::new(Mutex::new(vec![]));

						if command_build_blender_api {
							started_what.push(BuildStartedWhat::BlenderApi);
							let command_state = command_state.clone();
							let command_project_directory = command_project_directory.clone();
							let ended_what = ended_what.clone();
							let thread = spawn(move || {
								match subcommands::build_blender_api::subcommand(
									&subcommands::build_blender_api::Options {
										diff,
										force,
										project_directory: &command_project_directory,
									},
								) {
									Ok(result) => {
										let result = match &result {
									subcommands::build_blender_api::ResultKind::BlenderAPIAvailable => Some(BuildEndedWhat::BlenderApi),
									subcommands::build_blender_api::ResultKind::Nothing => None,
									subcommands::build_blender_api::ResultKind::ShaderPassesAvailable(passes) => Some(BuildEndedWhat::ShaderPasses{ passes: passes.to_vec() }),
								};

										if let Some(result) = result {
											let mut ended_what = ended_what.lock().unwrap();
											ended_what.push(result);
										}
									}
									Err(err) => {
										let mut state = command_state.write().unwrap();
										state.broadcast(&Event::Error { message: &err });
									}
								}
							});
							threads.push(thread);
						}

						if command_build_executable {
							started_what.push(BuildStartedWhat::Executable);
							let command_state = command_state.clone();
							let command_project_directory = command_project_directory.clone();
							let ended_what = ended_what.clone();
							let thread = spawn(move || {
								match subcommands::build_executable::subcommand(
									&subcommands::build_executable::Options {
										force,
										project_directory: &command_project_directory,
									},
								) {
									Ok(result) => {
										let result = match &result {
									subcommands::build_executable::ResultKind::ExecutableAvailable => Some(BuildEndedWhat::Executable),
									subcommands::build_executable::ResultKind::Nothing => None,
								};

										if let Some(result) = result {
											let mut ended_what = ended_what.lock().unwrap();
											ended_what.push(result);
										}
									}
									Err(err) => {
										let mut state = command_state.write().unwrap();
										state.broadcast(&Event::Error { message: &err });
									}
								}
							});
							threads.push(thread);
						}

						{
							let mut state = command_state.write().unwrap();
							state.broadcast(&Event::BuildStarted {
								what: &started_what,
							});
						}

						for thread in threads {
							thread.join().unwrap();
						}

						let mut state = command_state.write().unwrap();
						let ended_what = ended_what.lock().unwrap();
						state.broadcast(&Event::BuildEnded { what: &ended_what });
					}

					Command::GetBlenderApiPath => {
						let path = generators::blender_api::Generator::get_path();
						let mut state = command_state.write().unwrap();
						state.broadcast(&Event::BlenderApiPath { path: &path })
					}

					Command::GetExecutableSize => {
						match subcommands::build_executable::get_path(&command_project_directory) {
							Ok(size) => {
								let mut state = command_state.write().unwrap();
								state.broadcast(&Event::ExecutableSize { size });
							}
							Err(err) => {
								let mut state = command_state.write().unwrap();
								state.broadcast(&Event::Error {
									message: &err.to_string(),
								});
							}
						}
					}

					Command::SetBuildExecutable(SetBuildExecutableCommand { build_executable }) => {
						command_build_executable = build_executable;
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
					let _ = watcher_tx_command.send(Command::Build {
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
