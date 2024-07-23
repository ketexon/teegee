use std::io::Write;

use rustyline::{config::Configurer, history::DefaultHistory};

use crate::{g::{subprocess::{Subprocess, SubprocessFn, SubprocessInfo}, Game}, ipc::SwitchComputerMessage, rl::password::PasswordHelper};

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

		fn run(&self, g: &Game, args: Vec<String>) -> std::io::Result<()> {
			if args.len() != 1 {
				let _ = g.start_exe_from_path("help", vec!["ssh".into()]);
				return Ok(());
			}

			let host = &args[0];

			if let Some(computer) = g.find_computer_by_address(host) {
				let mut rl = rustyline::Editor::<PasswordHelper, DefaultHistory>::new()
					.map_err(std::io::Error::other)?;
				rl.set_auto_add_history(false);
				rl.set_color_mode(rustyline::ColorMode::Forced);

				rl.set_helper(Some(PasswordHelper(false)));
				let username = rl.readline("Username: ").unwrap_or(String::new());

				let mut guard = rl.set_cursor_visibility(false).map_err(std::io::Error::other)?;

				rl.set_helper(Some(PasswordHelper(true)));
				let password = rl.readline("Password: ").unwrap_or(String::new());

				guard.take();

				match computer.find_user(&username) {
					None => println!("Incorrect username or password"),
					Some(user) => {
						if password != user.password {
							println!("Incorrect password.");
						}
						else {
							g.connection.borrow_mut().write_message(crate::ipc::Message::SwitchComputer(SwitchComputerMessage {
								new_id: computer.id,
							}))?;

							g.current_computer().should_quit.set(true);
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