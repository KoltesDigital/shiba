use crate::build::{self, BuildEvent, BuildOptions, BuildTarget};
use crate::export::{self, ExportOptions, ExportOutput};
use crate::run::{self, RunOptions};
use crate::types::{Pass, ProjectDescriptor, Variable};
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
	Export {
		directory: String,
		output: ExportOutput,
		target: BuildTarget,
	},
	Run,
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
	Exported {
		path: &'a str,
	},
	Error {
		message: &'a str,
	},
	LibraryCompiled {
		path: &'a str,
	},
	Run {
		duration: f32,
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
		#[derive(Default)]
		struct BuildTargetArtifacts {
			path: Option<PathBuf>,
			project_descriptor: Option<ProjectDescriptor>,
			static_files: Option<Vec<PathBuf>>,
		}

		let mut watcher: RecommendedWatcher =
			Watcher::new(tx_watcher, Duration::from_secs_f32(0.3))
				.expect("Failed to create watcher.");
		watcher
			.watch(command_project_directory.clone(), RecursiveMode::Recursive)
			.expect("Failed to watch project directory");

		let mut executable_artifacts = BuildTargetArtifacts::default();
		let mut library_artifacts = BuildTargetArtifacts::default();

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

							match ProjectDescriptor::load(&command_project_directory, target) {
								Ok(project_descriptor) => {
									let mut event_listener = |event: BuildEvent| match event {
										BuildEvent::ExecutableCompiled(event) => {
											match event.get_size() {
												Ok(size) => {
													let path = event.path.to_string_lossy();

													let mut command_state =
														command_state.write().unwrap();
													command_state.broadcast(&Event {
														id: &command_id,
														kind: EventKind::ExecutableCompiled {
															path: &path,
															size,
														},
													});
												}
												Err(err) => {
													let mut command_state =
														command_state.write().unwrap();
													command_state.broadcast(&Event {
														id: &command_id,
														kind: EventKind::Error { message: &err },
													});
												}
											}
											executable_artifacts.path = Some(event.path);
										}

										BuildEvent::LibraryCompiled(event) => {
											{
												let path = event.path.to_string_lossy();

												let mut command_state =
													command_state.write().unwrap();
												command_state.broadcast(&Event {
													id: &command_id,
													kind: EventKind::LibraryCompiled {
														path: &path,
													},
												});
											}
											library_artifacts.path = Some(event.path);
										}

										BuildEvent::ShaderProvided(event) => {
											let mut command_state = command_state.write().unwrap();
											command_state.broadcast(&Event {
												id: &command_id,
												kind: EventKind::ShaderProvided {
													passes: &event.passes,
													target,
													variables: &event.variables,
												},
											});
										}

										BuildEvent::StaticFilesProvided(event) => match target {
											BuildTarget::Executable => {
												executable_artifacts.static_files =
													Some(event.paths.clone());
											}
											BuildTarget::Library => {
												library_artifacts.static_files =
													Some(event.paths.clone());
											}
										},
									};

									let result = build::build_duration(
										&BuildOptions {
											force: force.unwrap_or(false),
											project_descriptor: &project_descriptor,
											target,
										},
										&mut event_listener,
									);

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
											match target {
												BuildTarget::Executable => {
													executable_artifacts.path = None;
												}
												BuildTarget::Library => {
													library_artifacts.path = None;
												}
											};

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

									match target {
										BuildTarget::Executable => {
											executable_artifacts.project_descriptor =
												Some(project_descriptor);
										}
										BuildTarget::Library => {
											library_artifacts.project_descriptor =
												Some(project_descriptor);
										}
									};
								}

								Err(err) => {
									match target {
										BuildTarget::Executable => {
											executable_artifacts.project_descriptor = None;
										}
										BuildTarget::Library => {
											library_artifacts.project_descriptor = None;
										}
									};

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

						CommandKind::Export {
							directory,
							output,
							target,
						} => {
							let artifact = match target {
								BuildTarget::Executable => &executable_artifacts,
								BuildTarget::Library => &library_artifacts,
							};
							if let Some(build_path) = &artifact.path {
								match export::export(&ExportOptions {
									build_path: &build_path,
									directory: &PathBuf::from(directory),
									project_descriptor: artifact
										.project_descriptor
										.as_ref()
										.unwrap(),
									output,
									static_files: artifact.static_files.as_ref().unwrap(),
								}) {
									Ok(path) => {
										let path = path.to_string_lossy();

										let mut command_state = command_state.write().unwrap();
										command_state.broadcast(&Event {
											id: &command.id,
											kind: EventKind::Exported { path: &path },
										});
									}
									Err(err) => {
										let mut command_state = command_state.write().unwrap();
										command_state.broadcast(&Event {
											id: &command.id,
											kind: EventKind::Error { message: &err },
										});
									}
								};
							} else {
								let mut command_state = command_state.write().unwrap();
								command_state.broadcast(&Event {
									id: &command_id,
									kind: EventKind::Error {
										message: "Project has not been built.",
									},
								});
							}
						}

						CommandKind::Run => {
							if let Some(executable_path) = &executable_artifacts.path {
								match run::run_duration(&RunOptions {
									executable_path: &executable_path,
									project_directory: &command_project_directory,
								}) {
									Ok(duration) => {
										let mut command_state = command_state.write().unwrap();
										command_state.broadcast(&Event {
											id: &command.id,
											kind: EventKind::Run {
												duration: duration.as_secs_f32(),
											},
										});
									}
									Err(err) => {
										let mut command_state = command_state.write().unwrap();
										command_state.broadcast(&Event {
											id: &command.id,
											kind: EventKind::Error { message: &err },
										});
									}
								};
							} else {
								let mut command_state = command_state.write().unwrap();
								command_state.broadcast(&Event {
									id: &command_id,
									kind: EventKind::Error {
										message: "Project has not been built.",
									},
								});
							}
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
