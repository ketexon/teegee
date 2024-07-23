use std::fmt::Debug;

use super::Game;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct SubprocessInfo {
	pub name: Option<String>,
	pub help_text: Option<String>,
	pub description: Option<String>,
}

pub trait SubprocessFn {
	fn info(&self) -> SubprocessInfo {
		Default::default()
	}

	fn run(&self, g: &mut Game, args: Vec<String>) -> std::io::Result<()>;
}

pub type Subprocess = &'static dyn SubprocessFn;

impl Debug for dyn SubprocessFn {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let info = self.info();
		write!(
			f,
			"Subprocess {{ info: {info:?} }}"
		)
	}
}

pub mod fs;
pub mod sys;