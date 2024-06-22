use std::{collections::HashMap, fs::DirEntry, io::{stdin, stdout, Write}, path::PathBuf, str::FromStr};

use bitmask::bitmask;
use chrono::{DateTime, Local, Utc};

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
	pub children: Vec<FSNode<'a>>,
}

impl<'a> Dir<'a> {
	pub fn empty() -> Self {
		Self {
			children: Vec::new(),
		}
	}

	pub fn new<U: Into<Vec<FSNode<'a>>>>(children: U) -> Self {
		Self {
			children: children.into(),
		}
	}

	pub fn get_child<T: ToString>(&self, name: T) -> Option<&FSNode>{
		let name_string = name.to_string();
		self.children
			.binary_search_by(|entry| entry.name.cmp(&name_string))
			.ok()
			.map(|index| &self.children[index])
	}
}

pub struct Root<'a> {
	pub node: FSNode<'a>,
}

impl<'a> Root<'a> {
	pub fn new<U: Into<Vec<FSNode<'a>>>>(children: U) -> Self {
		Self {
			node: FSNode::dir(
				"",
				dateparser::parse("15 Jan 2023 00:00").expect("Could not parse date"),
				Dir::new(children.into())
			),
		}
	}

	pub fn get_node(&self, path: &Path) -> Option<&FSNode> {
		self.node.get_node(path)
	}
}

pub enum FSNodeContent<'a> {
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

type FSNodeDateTime = chrono::DateTime<Utc>;

pub struct FSNode<'a> {
	pub parent: Option<&'a FSNode<'a>>,
	pub name: String,
	pub date: FSNodeDateTime,
	pub content: FSNodeContent<'a>,

	pub owner_security: Security,
	pub other_security: Security,
}

impl<'a> FSNode<'a> {
	pub fn dir<T: ToString>(name: T, date: FSNodeDateTime, dir: Dir<'a>) -> Self {
		Self {
			parent: None,
			name: name.to_string(),
			date,
			content: FSNodeContent::Dir(dir),

			owner_security: SecurityMode::RW.into(),
			other_security: SecurityMode::Read.into(),
		}
	}

	pub fn is_dir(&self) -> bool {
		matches!(&self.content, FSNodeContent::Dir(_))
	}

	pub fn is_file(&self) -> bool {
		matches!(&self.content, FSNodeContent::File(_))
	}

	pub fn file<T: ToString>(name: T, date: FSNodeDateTime, file: File) -> Self {
		Self {
			parent: None,
			name: name.to_string(),
			date,
			content: FSNodeContent::File(file),

			owner_security: SecurityMode::RW.into(),
			other_security: SecurityMode::Read.into(),
		}
	}

	pub fn get_node(&self, path: &Path) -> Option<&FSNode> {
		match &path.0[..] {
			[a, b @ ..] => match &self.content {
				FSNodeContent::Dir(dir) => dir.get_child(a).and_then(|child| child.get_node(&Path::new(b))),
				_ => None,
			},
			_ => Some(self)
		}
	}
}

type Subprocess<'a> = &'a dyn Fn(&mut Game, Vec<String>) -> std::io::Result<()>;

pub struct Game<'a> {
	pub should_quit: bool,
	pub cwd: Path,
	pub drive: Option<String>,
	pub root: Root<'a>,
	pub subprocesses: HashMap<String, Subprocess<'a>>,
}

pub fn parse_command<T: IntoIterator<Item = char>>(command: T) -> Vec<String> {
	let mut in_string = false;
	let mut escaping = false;

	let mut args = Vec::<String>::new();
	let mut cur_arg = String::new();

	for ch in command.into_iter() {
		if escaping {
			cur_arg.push(ch);
			escaping = false;
		}
		else {
			match ch {
				'\\' => escaping = true,
				'"' => in_string = !in_string,
				ch if ch.is_whitespace() && !in_string => {
					if cur_arg.len() > 0 {
						args.push(cur_arg.clone());
					}
					cur_arg.clear();
				},
				ch => cur_arg.push(ch)
			};
		}
	}

	if cur_arg.len() > 0 {
		args.push(cur_arg);
	}

	return args;
}

pub fn cmd(mut g: &mut Game, args: Vec<String>) -> std::io::Result<()> {
	while !g.should_quit {
        print!("{}> ", g.cwd.format(&g.drive));
        stdout().flush()?;

        let line = {
            let mut buf = String::new();
            stdin().read_line(&mut buf)?;
            buf.trim().to_string()
        };

        let args = parse_command(line.chars());

        if args.len() > 0 {
            let proc_name = args[0].clone();

            if let Some(subprocess) = g.subprocesses.get(&proc_name).cloned() {
                let argv: Vec<String> = args[1..].into();
                subprocess(&mut g, argv);
            }
            else {
                println!("Unknown process \"{}\"", proc_name);
            }
        }
    }
	Ok(())
}

pub fn logout(game: &mut Game, args: Vec<String>) -> std::io::Result<()> {
	game.should_quit = true;
	Ok(())
}

pub fn ls(game: &mut Game, args: Vec<String>) -> std::io::Result<()>{
	if let Some(entry) = game.root.get_node(&game.cwd) {
		if let FSNodeContent::Dir(dir) = &entry.content {
			let mut v: Vec<&FSNode> = dir.children.iter().collect();
			v.sort_unstable_by(|a, b| match (&a.content, &b.content) {
				(FSNodeContent::Dir(_), FSNodeContent::Dir(_))
				| (FSNodeContent::File(_), FSNodeContent::File(_)) => a.name.cmp(&b.name),
				(FSNodeContent::Dir(_), FSNodeContent::File(_)) => std::cmp::Ordering::Less,
				(FSNodeContent::File(_), FSNodeContent::Dir(_)) => std::cmp::Ordering::Greater,
			});

			let rows = v.iter().map(|node| [
				format!(
					"{}{}{}",
					if node.is_dir() { 'd' } else { '-' },
					node.owner_security,
					node.other_security
				),
				format!("{}", node.date),
				if let FSNodeContent::File(f) = &node.content {
					f.content.len().to_string()
				} else {
					"".to_string()
				},
				node.name.clone(),
			]).collect::<Vec<[String; 4]>>();

			let column_sizes =
				rows.iter()
				.map(|cols| cols.each_ref().map(|col| col.len()))
				.fold(
					[8, 8, 8, 8],
					|acc, e| [
						std::cmp::max(acc[0], e[0]),
						std::cmp::max(acc[1], e[1]),
						std::cmp::max(acc[2], e[2]),
						std::cmp::max(acc[3], e[3]),
					]
				)
				.map(|x| x + 2);
			let column_pad_right = [
				true,
				false,
				false,
				true
			];
			for columns in rows {
				for ((column, size), pad_right) in columns.iter().zip(column_sizes).zip(column_pad_right) {
					if pad_right {
						print!("{column:<size$}");
					}
					else{
						print!("{column:>size$} ");
					}
				}
				println!();
			}
		}
	}
	Ok(())
}

pub fn cd(game: &mut Game, args: Vec<String>) -> std::io::Result<()> {
	if args.len() != 1 {
		println!("Expected 1 argument.");
		return Ok(());
	}
	let subdir = Path::parse(&game.cwd, args[0].clone());
	if let Some(FSNode { content: FSNodeContent::Dir(_), .. }) = game.root.get_node(&subdir) {
		game.cwd = subdir;
	}
	else {
		println!("No such directory \"{}\".", subdir.format(&game.drive));
	}
	Ok(())
}

pub fn cat(game: &mut Game, args: Vec<String>) -> std::io::Result<()> {
	if let Some(FSNode { content: FSNodeContent::Dir(d), .. }) = game.root.get_node(&game.cwd) {
		let mut buf = String::new();
		for file in args {
			if let Some(FSNode { content: FSNodeContent::File(f), .. }) = d.get_child(&file) {
				buf += &f.content;
				buf.push('\n');
			}
			else {
				println!("Cannot find file \"{file}\"");
				return Ok(());
			}
			print!("{buf}");
			std::io::stdout().flush().expect("Could not flush stdio");
		}
	}
	Ok(())
}

impl<'a> Default for Game<'a> {
	fn default() -> Self {
		Self {
			should_quit: false,
			cwd: Path::default(),
			drive: Some("C".to_string()),
			root: Root::new(vec![
				FSNode::file("hello", dateparser::parse("12 Jan 2024 12:30").unwrap(), File::new("there")),
				FSNode::dir("test", dateparser::parse("14 Jan 2024 12:30").unwrap(), Dir::new([
					FSNode::file("wow", dateparser::parse("12 Jan 2024 12:30").unwrap(), File::new("there"))
				]))
			]),
			subprocesses: HashMap::from([
				("cmd".to_string(), &cmd as Subprocess<'a>),
				("logout".to_string(), &logout as Subprocess<'a>),
				("ls".to_string(), &ls as Subprocess<'a>),
				("cd".to_string(), &cd as Subprocess<'a>),
				("cat".to_string(), &cat as Subprocess<'a>),
			]),
		}
	}
}

fn test(g: &mut Game){

}

impl<'a> Game<'a> {
	pub fn start_process<U: Into<Vec<String>>>(&mut self, name: &str, args: U) -> std::io::Result<()> {
		self.subprocesses.get(name)
			.cloned()
			.ok_or(std::io::Error::other("No process found"))
			.and_then(|f| f(self, args.into()))
	}
}