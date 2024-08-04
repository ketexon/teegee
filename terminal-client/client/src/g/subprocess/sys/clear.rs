use crossterm::{cursor, terminal, ExecutableCommand};

use crate::g::subprocess::{Subprocess, SubprocessFn};

pub const CLEAR: Subprocess = {
	struct Clear;

	impl SubprocessFn for Clear {
    fn run(&self, _g: &crate::g::Game, _args: Vec<String>) -> std::io::Result<()> {
			std::io::stdout()
				.execute(cursor::MoveTo(0, 0))?
				.execute(terminal::Clear(terminal::ClearType::All))?
				.execute(terminal::Clear(terminal::ClearType::Purge))?;
			Ok(())
    }
}

	&Clear
};
