use std::collections::{HashMap, VecDeque};

pub mod computer;
pub mod fs;
pub mod subprocess;

pub use computer::Computer;
use computer::{ComputerBuilder, User};
use fs::{File, Node, Path};
use subprocess::SubprocessFn;

use crate::{date, ipc, path, util::wait_for_input};

pub struct Game<'a> {
	pub connection: &'a mut Box<dyn ipc::Connection>,
	pub computers: Vec<Computer>,
	pub current_computer_index: usize,

	computer_address_map: HashMap<String, usize>,
	process_queue: VecDeque<(String, Vec<String>)>,
}

impl<'a> Game<'a> {
	pub fn new(connection: &'a mut Box<dyn ipc::Connection>) -> Self {
		let default_exes =
			std::iter::empty()
			.chain(subprocess::sys::DEFAULT)
			.chain(subprocess::fs::DEFAULT)
			.cloned();

		let computers = vec![
			ComputerBuilder::new()
				.name("Plasma XQ9")
				.users([
					User::new("root", "123456"),
				])
				.current_user("root")
				.address("192.168.0.1")
				.with_path("bin".to_string())
				.add_dir(&path![], "bin", date!["12 Jan 2024 12:30"])
				.add_file(&path![], "hello1", date!["12 Jan 2024 12:30"], File::new("there"))
				.add_exes_same_date(&path!["bin"], date!["12 Jan 2024 12:30"], default_exes.clone())
				.build(),
			ComputerBuilder::new()
				.name("Computer 1")
				.address("192.168.0.1")
				.with_path("bin".to_string())
				.add_dir(&path![], "bin", date!["12 Jan 2024 12:30"])
				.add_file(&path![], "hello2", date!["12 Jan 2024 12:30"], File::new("there"))
				.add_exes_same_date(&path!["bin"], date!["12 Jan 2024 12:30"], default_exes.clone())
				.build(),
		];
		let computer_address_map: HashMap<String, usize> =
			computers.iter()
			.enumerate()
			.map(|(i, c)| (c.address.clone(), i))
			.collect();

		Self {
			connection,
			computers,
			current_computer_index: 0,
			computer_address_map,
			process_queue: Default::default(),
		}
	}

	pub fn current_computer(&self) -> &Computer {
		&self.computers[self.current_computer_index]
	}

	pub fn current_computer_mut(&mut self) -> &mut Computer {
		&mut self.computers[self.current_computer_index]
	}

	pub fn start_exe_from_path<U: Into<Vec<String>>>(&mut self, name: &str, args: U) -> Result<std::io::Result<()>, fs::FsError> {
		self.current_computer().which_node(&name.to_string())
			.ok_or(fs::FsError::PathIsNotExecutable)
			.and_then(|node| self.start_exe(node, args))
	}

	pub fn start_exe<U: Into<Vec<String>>>(&mut self, node: Node, args: U) -> Result<std::io::Result<()>, fs::FsError> {
		node.as_exe()
			.ok_or(fs::FsError::PathIsNotExecutable)
			.map(|node| node.run(self, args.into()))
	}

	pub fn queue_process<U: Into<Vec<String>>>(&mut self, name: &str, args: U) {
		self.process_queue.push_back((name.to_string(), args.into()));
	}

	pub fn get_queued_process(&mut self) -> Option<(String, Vec<String>)> {
		self.process_queue.pop_front()
	}

	#[allow(dead_code)]
	pub fn find_computer_by_address(&self, addr: &String) -> Option<&Computer> {
		self.computer_address_map.get(addr).map(|i| &self.computers[*i])
	}

	#[allow(dead_code)]
	pub fn find_computer_by_address_mut(&mut self, addr: &String) -> Option<&mut Computer> {
		self.computer_address_map.get(addr).map(|i| &mut self.computers[*i])
	}

	pub fn change_computers_by_address(&mut self, addr: &String) -> bool {
		if let Some(index) = self.computer_address_map.get(addr) {
			self.current_computer_index = *index;
			return true;
		}
		false
	}
}