use std::collections::HashMap;

pub mod computer;
pub mod fs;
pub mod subprocess;

pub use computer::Computer;
pub use subprocess::Subprocess;

pub struct Game<'a> {
	pub should_quit: bool,
	pub c: Computer<'a>
}


impl<'a> Default for Game<'a> {
	fn default() -> Self {
		Self {
			should_quit: false,
			c: Computer {
				cwd: fs::Path::default(),
				drive: Some("C".to_string()),
				root: fs::Root::new(vec![
					fs::Node::file("hello", dateparser::parse("12 Jan 2024 12:30").unwrap(), fs::File::new("there")),
					fs::Node::dir("test", dateparser::parse("14 Jan 2024 12:30").unwrap(), fs::Dir::new([
						fs::Node::file("wow", dateparser::parse("12 Jan 2024 12:30").unwrap(), fs::File::new("there"))
					]))
				]),
				subprocesses: HashMap::from([
					("cmd".to_string(), &subprocess::sys::cmd as Subprocess<'a>),
					("logout".to_string(), &subprocess::sys::logout as Subprocess<'a>),
					("ls".to_string(), &subprocess::fs::ls as subprocess::Subprocess<'a>),
					("cd".to_string(), &subprocess::fs::cd as subprocess::Subprocess<'a>),
					("cat".to_string(), &subprocess::fs::cat as subprocess::Subprocess<'a>),
					("ssh".to_string(), &subprocess::sys::ssh as Subprocess<'a>),
				]),
			}
		}
	}
}

impl<'a> Game<'a> {
	pub fn start_process<U: Into<Vec<String>>>(&mut self, name: &str, args: U) -> std::io::Result<()> {
		self.c.subprocesses.get(name)
			.cloned()
			.ok_or(std::io::Error::other("No process found"))
			.and_then(|f| f(self, args.into()))
	}
}