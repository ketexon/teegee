use std::collections::BinaryHeap;

use crate::g::{fs::NodeContent, subprocess::{Subprocess, SubprocessFn, SubprocessInfo}, Game};

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

		fn run(&self, g: &Game, args: Vec<String>) -> std::io::Result<()> {
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

				g.current_computer().exes()
					.iter()
					.map(|node| (node.borrow().name.clone(), node.as_exe().unwrap().info()))
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