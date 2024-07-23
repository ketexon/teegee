use std::{collections::BinaryHeap, io::Write};

use crate::{g::{self, fs::{FsError, Node, NodeContent, Path}, subprocess::SubprocessInfo, Game}, ipc::{Connection, SwitchComputerMessage}, path};

use super::{Subprocess, SubprocessFn};

pub const DEFAULT: &[(&str, Subprocess)] = &[
	("cmd", CMD),
	("logout", LOGOUT),
	("ssh", SSH),
	("help", HELP),
];

fn parse_command<T: IntoIterator<Item = char>>(command: T) -> Vec<String> {
	let mut in_string = false;
	let mut escaping = false;

	let mut args = Vec::<String>::new();
	let mut cur_arg = String::new();

	for ch in command.into_iter() {
		if escaping {
			cur_arg.push(ch);
			escaping = false;
		}
		else {
			match ch {
				'\\' => escaping = true,
				'"' => in_string = !in_string,
				ch if ch.is_whitespace() && !in_string => {
					if cur_arg.len() > 0 {
						args.push(cur_arg.clone());
					}
					cur_arg.clear();
				},
				ch => cur_arg.push(ch)
			};
		}
	}

	if cur_arg.len() > 0 {
		args.push(cur_arg);
	}

	return args;
}

const DEFAULT_PS1: &str = "\\u@\\H \\w$ ";

pub const CMD: Subprocess = {
	pub struct Cmd;
	impl SubprocessFn for Cmd {
		fn run(&self, g: &mut Game, args: Vec<String>) -> std::io::Result<()> {
			while !g.current_computer().should_quit {
				let ps1 = g.current_computer().env
					.get(&"PS1".to_string()).map(String::as_str)
					.unwrap_or(DEFAULT_PS1);

				print!(
					"{}",
					ps1.replace("\\u", &g.current_computer().current_user)
						.replace("\\H", &g.current_computer().name)
						.replace("\\w", &g.current_computer().cwd.to_string())
				);
				std::io::stdout().flush()?;

				let line = {
					let mut buf = String::new();
					std::io::stdin().read_line(&mut buf)?;
					buf.trim().to_string()
				};

				let args = parse_command(line.chars());

				if args.len() > 0 {
					let proc_name = args[0].clone();

					if let Err(e) = g.start_exe_from_path(&proc_name, &args[1..]) {
						match e {
							FsError::PathIsNotExecutable => {
								println!("Could not find process \"{}\"\nType \"help\" to list all processes.", proc_name);
							}
							_ => ()
						}
					}
				}
			}
			Ok(())
		}
	}
	&Cmd
};

pub const LOGOUT: Subprocess = {
	struct Logout;

	impl SubprocessFn for Logout {
		fn run(&self, game: &mut Game, args: Vec<String>) -> std::io::Result<()> {
			game.current_computer_mut().should_quit = true;
			Ok(())
		}
	}

	&Logout
};

pub const WHICH: Subprocess = {
	struct Which;

	impl SubprocessFn for Which {
		fn run(&self, g: &mut Game, args: Vec<String>) -> std::io::Result<()> {
			if args.len() != 1 {

				return Ok(());
			}
			if let Some(path) = g.current_computer().which_path(&args[0]) {
				println!("{path}");
			}
			else {
				println!("Could not find executable: \"{}\"", args[0])
			}
			return Ok(())
		}
	}

	&Which
};

pub const SSH: Subprocess = {
	struct Ssh;
	impl SubprocessFn for Ssh {
		fn info(&self) -> SubprocessInfo {
			SubprocessInfo {
				name: Some("Secure Shell".into()),
				description: Some("creates a remote shell to another computer.".into()),
				help_text: Some(concat!(
					"ssh host\n"
				).into()),
			}
		}

		fn run(&self, g: &mut Game, args: Vec<String>) -> std::io::Result<()> {
			if args.len() != 1 {
				let _ = g.start_exe_from_path("help", vec!["ssh".into()]);
				return Ok(());
			}

			let host = &args[0];

			if let Some(computer) = g.find_computer_by_address(host) {
				print!("Username: ");
				std::io::stdout().flush()?;

				let mut username = String::new();
				std::io::stdin().read_line(&mut username)?;

				print!("Password: ");
				std::io::stdout().flush()?;

				let mut password_buf = String::new();
				let password = match std::io::stdin().read_line(&mut password_buf) {
					Ok(0) => password_buf,
					Ok(_) => {
						if password_buf.ends_with("\r\n") {
							password_buf[..password_buf.len() - 2].to_string()
						}
						else {
							password_buf[..password_buf.len() - 1].to_string()
						}
					},
					_ => {
						return Ok(());
					}
				};

				match computer.find_user(&username) {
					None => println!("Incorrect username or password"),
					Some(user) => {
						if password != user.password {
							println!("Incorrect password.");
						}
						else {
							g.connection.write_message(crate::ipc::Message::SwitchComputer(SwitchComputerMessage {
								new_id: computer.id,
							}))?;

							g.current_computer_mut().should_quit = true;
							println!("Successfully connected");
							g.change_computers_by_address(&host);
							g.queue_process("cmd", []);
						}
					}
				}
			}
			else {
				println!("Host does not exist.");
				return Ok(());
			}

			Ok(())
		}
	}
	&Ssh
};

pub const HELP: Subprocess = {
	pub struct Help;
	impl SubprocessFn for Help {
		fn info(&self) -> SubprocessInfo {
			SubprocessInfo {
				name: Some("Help".into()),
				description: Some("Shows helpful information about executables.".into()),
				help_text: Some(
					concat!(
						"help [exe_name]\n",
						"\tShows useful information about an executable.\n",
						"\tIf exe_name is not specified, shows all executables."
					).into()
				),
				..Default::default()
			}
		}

		fn run(&self, g: &mut Game, args: Vec<String>) -> std::io::Result<()> {
			if args.len() == 1 {
				let subprocess_name = args.get(0).unwrap();
				match g.current_computer().which_node(&subprocess_name) {
					Some(node) => {
						let data = node.borrow();
						match data.content {
							NodeContent::Executable(exe) => {
								let info = exe.info();
								let name = if let Some(name) = info.name {
									name
								} else {
									data.name.clone()
								};

								if let Some(desc) = info.description {
									println!("{name} -- {desc}")
								}
								else {
									println!("{name}")
								}

								if let Some(help) = info.help_text {
									println!("{help}")
								}
								else {
									println!("This executable has no help documentation.");
								}
							},
							_ => println!("Could not find executable \"{}\"", subprocess_name),
						}
					}
					None => println!("Could not find executable \"{}\"", subprocess_name),
				};
			}
			else {
				println!("{}", self.info().help_text.unwrap());

				let names = g.current_computer().parsed_path()
					.iter()
					.filter_map(|entry|
						g.current_computer().root
							.get_dir(&Path::parse(&path![], entry))
					)
					.flat_map(|dir|
						dir.children
					)
					.filter_map(|node| {
						let data = node.borrow();
						match data.content {
							NodeContent::Executable(exe) => Some((data.name.clone(), exe.info())),
							_ => None,
						}
					})
					.collect::<BinaryHeap<(String, SubprocessInfo)>>()
					.iter()
					.rev()
					.for_each(|(name, info)| {
						if let Some(desc) = info.description.clone() {
							println!("{name} -- {desc}");
						}
						else {
							println!("{name}");
						}
					});
			}


			Ok(())
		}
	}
	&Help
};