use crate::g::{subprocess::{Subprocess, SubprocessFn}, Game};

pub const WHICH: Subprocess = {
	struct Which;

	impl SubprocessFn for Which {
		fn run(&self, g: &Game, args: Vec<String>) -> std::io::Result<()> {
			if args.len() != 1 {

				return Ok(());
			}
			if let Some(path) = g.current_computer().which_path(&args[0]) {
				println!("{path}");
			}
			else {
				println!("Could not find executable: \"{}\"", args[0])
			}
			Ok(())
		}
	}

	&Which
};
