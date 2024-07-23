
use crate::g::{subprocess::{Subprocess, SubprocessFn}, Game};

pub const LOGOUT: Subprocess = {
	struct Logout;

	impl SubprocessFn for Logout {
		fn run(&self, game: &Game, _args: Vec<String>) -> std::io::Result<()> {
			game.current_computer().should_quit.set(true);
			Ok(())
		}
	}

	&Logout
};
