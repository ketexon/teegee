use bitmask::bitmask;

pub struct Path(pub Vec<String>);

impl Default for Path {
	fn default() -> Self {
		Self(Vec::new())
	}
}

impl Path {
	pub fn new<T: Into<Vec<String>>>(path: T) -> Self {
		Self(path.into())
	}

	pub fn format<T: std::fmt::Display>(&self, drive: &Option<T>) -> String {
		match drive {
			Some(d) => format!("{}:/{}", d, self.0.join("/")),
			None => format!("/{}", self.0.join("/")),
		}
	}

	pub fn parse(cwd: &Path, relative: String) -> Self {
		let mut new_path = if relative.starts_with('/') {
			Vec::new()
		} else{
			cwd.0.clone()
		};

		for sub in relative.split('/') {
			if sub.len() == 0 || sub == "." { continue; }
			else if sub == ".." {
				new_path.pop();
			}
			else {
				new_path.push(sub.to_string());
			}
		}
		Self(new_path)
	}
}

pub struct File {
	pub content: String,
}

impl File {
	pub fn new<U: ToString>(content: U) -> Self {
		Self {
			content: content.to_string()
		}
	}
}

pub struct Dir<'a> {
	pub children: Vec<Node<'a>>,
}

impl<'a> Dir<'a> {
	pub fn empty() -> Self {
		Self {
			children: Vec::new(),
		}
	}

	pub fn new<U: Into<Vec<Node<'a>>>>(children: U) -> Self {
		Self {
			children: children.into(),
		}
	}

	pub fn get_child<T: ToString>(&self, name: T) -> Option<&Node>{
		let name_string = name.to_string();
		self.children
			.binary_search_by(|entry| entry.name.cmp(&name_string))
			.ok()
			.map(|index| &self.children[index])
	}
}

pub struct Root<'a> {
	pub node: Node<'a>,
}

impl<'a> Root<'a> {
	pub fn new<U: Into<Vec<Node<'a>>>>(children: U) -> Self {
		Self {
			node: Node::dir(
				"",
				dateparser::parse("15 Jan 2023 00:00").expect("Could not parse date"),
				Dir::new(children.into())
			),
		}
	}

	pub fn get_node(&self, path: &Path) -> Option<&Node> {
		self.node.get_node(path)
	}
}

pub enum NodeContent<'a> {
	File(File),
	Dir(Dir<'a>),
}

bitmask! {
	pub mask Security: u8 where flags SecurityMode {
		Read 	= 1,
		Write 	= 2,
		Execute = 4,
		RW 		= 1 | 2,
		All 	= 7,
	}
}

impl std::fmt::Display for Security {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}{}{}",
			if self.contains(SecurityMode::Read) { 'r' } else { '-' },
			if self.contains(SecurityMode::Write) { 'w' } else { '-' },
			if self.contains(SecurityMode::Execute) { 'x' } else { '-' }
		)
	}
}

type NodeDateTime = chrono::DateTime<chrono::Utc>;

pub struct Node<'a> {
	pub parent: Option<&'a Node<'a>>,
	pub name: String,
	pub date: NodeDateTime,
	pub content: NodeContent<'a>,

	pub owner_security: Security,
	pub other_security: Security,
}

impl<'a> Node<'a> {
	pub fn dir<T: ToString>(name: T, date: NodeDateTime, dir: Dir<'a>) -> Self {
		Self {
			parent: None,
			name: name.to_string(),
			date,
			content: NodeContent::Dir(dir),

			owner_security: SecurityMode::RW.into(),
			other_security: SecurityMode::Read.into(),
		}
	}

	pub fn is_dir(&self) -> bool {
		matches!(&self.content, NodeContent::Dir(_))
	}

	pub fn is_file(&self) -> bool {
		matches!(&self.content, NodeContent::File(_))
	}

	pub fn file<T: ToString>(name: T, date: NodeDateTime, file: File) -> Self {
		Self {
			parent: None,
			name: name.to_string(),
			date,
			content: NodeContent::File(file),

			owner_security: SecurityMode::RW.into(),
			other_security: SecurityMode::Read.into(),
		}
	}

	pub fn get_node(&self, path: &Path) -> Option<&Node> {
		match &path.0[..] {
			[a, b @ ..] => match &self.content {
				NodeContent::Dir(dir) => dir.get_child(a).and_then(|child| child.get_node(&Path::new(b))),
				_ => None,
			},
			_ => Some(self)
		}
	}
}