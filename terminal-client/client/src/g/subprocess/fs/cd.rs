use crate::g::{subprocess::{Subprocess, SubprocessFn}, Game};
use crate::g::fs;

pub const CD: Subprocess = {
	struct Cd;
	impl SubprocessFn for Cd {
		fn run(&self, g: &Game, args: Vec<String>) -> std::io::Result<()> {
			if args.len() != 1 {
				println!("Expected 1 argument.");
				return Ok(());
			}
			let subdir = fs::Path::parse(&g.current_computer().cwd.borrow(), &args[0]);
			if let Some(node) = g.current_computer().root.get_node(&subdir) {
				if node.is_dir() {
					g.current_computer().cwd.replace(subdir);
				}
				else{
					println!("Path is not a directory \"{}\".", subdir);
				}
			}
			else {
				println!("No such directory \"{}\".", subdir);
			}
			Ok(())
		}
	}
	&Cd
};