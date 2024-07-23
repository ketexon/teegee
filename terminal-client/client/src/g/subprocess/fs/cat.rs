use std::io::Write;

use crate::g::{fs::Path, subprocess::{Subprocess, SubprocessFn}, Game};

pub const CAT: Subprocess = {
	struct Cat;
	impl SubprocessFn for Cat {
		fn run(&self, g: &Game, args: Vec<String>) -> std::io::Result<()> {
			let cwd = g.current_computer().cwd.borrow().clone();

			let mut buf = String::new();
			for file in args {
				if let Some(node) = g.current_computer().root.get_node(&Path::parse(&cwd, &file)) {
					if let Some(f) = node.as_file() {
						buf += &f.content;
						buf.push('\n');
					}
					else {
						println!("Path \"{file}\" is not a file.");
					}
				}
				else {
					println!("File \"{file}\" does not exist.");
					return Ok(());
				}
				std::io::stdout().flush().expect("Could not flush stdio");
			}
			print!("{buf}");

			Ok(())
		}
	}
	&Cat
};