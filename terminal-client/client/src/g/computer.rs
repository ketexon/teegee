use std::collections::HashMap;
use bevy_reflect::Reflect;

use crate::path;
use crate::util::wait_for_input;

use super::fs::{Dir, Executable, File, Node, NodeData, NodeDateTime, Path, Root};
use super::subprocess::SubprocessFn;

#[repr(u32)]
#[derive(Reflect, Debug, Clone, Copy)]
pub enum ComputerId {
	First = 0,
	Second = 1,
}

type ComputerAddress = String;

pub struct User {
	pub name: String,
	pub password: String,
}

impl User {
	pub fn new<T: Into<String>, U: Into<String>>(name: T, password: U) -> Self {
		Self {
			name: name.into(),
			password: password.into(),
		}
	}
}

pub struct Computer {
	pub should_quit: bool,

	pub name: String,
	pub users: Vec<User>,
	pub current_user: String,
	pub id: ComputerId,
	pub address: ComputerAddress,

	pub cwd: Path,
	pub drive: Option<String>,
	pub root: Root,
	pub env: HashMap<String, String>,
}

impl Computer {
	pub fn path(&self) -> String {
		self.env.get(&"path".to_string()).cloned().unwrap_or_default()
	}

	pub fn parsed_path(&self) -> Vec<String> {
		self.path().split(';').map(|s| s.to_string()).collect()
	}

	pub fn which(&self, exe: &str) -> Option<(Path, Node)> {
		let path = self.parsed_path();
		path
			.iter()
			.find_map(|entry| {
				let exe_path = Path::parse(&Path::default(), entry).join(&path![exe.clone()]);
				self.root.get_node(&exe_path)
					.and_then(|node| if node.is_exe() { Some(node) } else { None })
					.map(|node| (exe_path, node))
			})
	}

	pub fn which_node(&self, exe: &str) -> Option<Node> {
		self.which(exe).map(|v| v.1)
	}

	pub fn which_path(&self, exe: &str) -> Option<Path> {
		self.which(exe).map(|v| v.0)
	}

	pub fn find_user(&self, name: &str) -> Option<&User> {
		return self.users.iter().find(|user| user.name == name)
	}
}

impl Default for Computer {
	fn default() -> Self {
		Self {
			should_quit: false,

			name: Default::default(),
			users: Default::default(),
			current_user: Default::default(),
			id: ComputerId::First,
			address: Default::default(),

			cwd: Default::default(),
			drive: Some("C".to_string()),
			root: Root::new([]),

			env: HashMap::from([
				("PS1".to_string(), "\\u@\\H \\w$ ".to_string()),
			]),
		}
	}
}

pub struct ComputerBuilder(Computer);

impl ComputerBuilder {
	pub fn new() -> Self { Self(Default::default()) }

	pub fn name<T: ToString>(mut self, name: T) -> Self {
		self.0.name = name.to_string();
		self
	}

	pub fn users<T: Into<Vec<User>>>(mut self, users: T) -> Self {
		self.0.users = users.into();
		self
	}

	pub fn add_user<T: Into<User>>(mut self, user: T) -> Self {
		let user = user.into();
		let user_name = user.name.clone();
		self.0.users.push(user);
		if self.0.users.len() == 0 {
			self.current_user(user_name)
		}
		else {
			self
		}
	}

	pub fn current_user<T: ToString>(mut self, current_user: T) -> Self {
		self.0.current_user = current_user.to_string();
		self
	}

	pub fn id(mut self, id: ComputerId) -> Self {
		self.0.id = id;
		self
	}

	pub fn address<T: Into<ComputerAddress>>(mut self, address: T) -> Self {
		self.0.address = address.into();
		self
	}

	pub fn cwd(mut self, cwd: Path) -> Self {
		self.0.cwd = cwd;
		self
	}

	pub fn drive(mut self, drive: String) -> Self {
		self.0.drive = Some(drive);
		self
	}

	pub fn add_fs_node(self, root: &Path, node: Node) -> Self {
		self.0.root.node.add_node(root, node).expect("Could not add fs node.");
		self
	}

	pub fn add_file<T: ToString>(self, root: &Path, name: T, date: NodeDateTime, file: File) -> Self {
		self.add_fs_node(root, Node::file(name, date, file))
	}

	pub fn add_dir<T: ToString>(self, root: &Path, name: T, date: NodeDateTime) -> Self {
		self.add_fs_node(root, Node::dir(name, date, Dir::empty()))
	}

	pub fn add_exe<T: ToString>(self, root: &Path, name: T, date: NodeDateTime, subprocess: &'static dyn SubprocessFn) -> Self {
		self.add_fs_node(root, Node::exe(name, date, subprocess))
	}

	pub fn add_exes<'a, T: ToString, It>(self, root: &Path, exes: It) -> Self
		where It: IntoIterator<Item=(T, NodeDateTime, &'static dyn SubprocessFn)>
	{
		exes.into_iter().fold(self,
			|s, (name, date, subprocess)|
			s.add_exe(&root, name, date, subprocess)
		)
	}

	pub fn add_exes_same_date<'a, T: ToString, It>(self, root: &Path, date: NodeDateTime, exes: It) -> Self
		where It: IntoIterator<Item=(T, &'static dyn SubprocessFn)>
	{
		exes.into_iter().fold(self,
			|s, (name, subprocess)|
			s.add_exe(&root, name, date, subprocess)
		)
	}

	pub fn with_path(mut self, path: String) -> Self {
		self.0.env.entry("path".to_string())
			.and_modify(|p| {
				p.push(';');
				p.push_str(path.as_str());
			})
			.or_insert(path);
		self
	}

	pub fn ps1(mut self, ps1: String) -> Self {
		self.0.env.insert("PS1".to_string(), ps1);
		self
	}

	pub fn build(self) -> Computer {
		self.0
	}
}