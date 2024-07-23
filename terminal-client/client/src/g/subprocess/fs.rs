use std::io::Write;

use crate::g::{fs::{self, Path}, Game};
use super::{Subprocess, SubprocessFn, SubprocessInfo};

pub const DEFAULT: &[(&str, Subprocess)] = &[
	("ls", LS),
	("cd", CD),
	("cat", CAT),
];

pub const LS: Subprocess = {
	struct Ls;
	impl SubprocessFn for Ls {
		fn run(&self, g: &mut Game, args: Vec<String>) -> std::io::Result<()> {
			if let Some(entry) = g.current_computer().root.get_node(&g.current_computer().cwd) {
				if let Some(dir) = &entry.as_dir() {
					use fs::NodeContent;

					let mut v: Vec<&fs::Node> = dir.children.iter().collect();
					v.sort_unstable_by(|a, b| match (&a.borrow().content, &b.borrow().content) {
						(NodeContent::Dir(_), NodeContent::Dir(_))
						| (
							NodeContent::File(_) | NodeContent::Executable(_),
							NodeContent::File(_) | NodeContent::Executable(_)
						) => a.borrow().name.cmp(&b.borrow().name),
						(NodeContent::Dir(_), NodeContent::File(_) | NodeContent::Executable(_)) => std::cmp::Ordering::Less,
						(NodeContent::File(_) | NodeContent::Executable(_), NodeContent::Dir(_)) => std::cmp::Ordering::Greater,
					});

					let rows = v.iter().map(|node| [
						format!(
							"{}{}{}",
							if node.is_dir() { 'd' } else { '-' },
							node.borrow().owner_security,
							node.borrow().other_security
						),
						format!("{}", node.borrow().date),
						if let fs::NodeContent::File(f) = &node.borrow().content {
							f.content.len().to_string()
						} else {
							"".to_string()
						},
						node.borrow().name.clone(),
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
	}

	&Ls
};

pub const CD: Subprocess = {
	struct Cd;
	impl SubprocessFn for Cd {
		fn run(&self, g: &mut Game, args: Vec<String>) -> std::io::Result<()> {
			if args.len() != 1 {
				println!("Expected 1 argument.");
				return Ok(());
			}
			let subdir = fs::Path::parse(&g.current_computer().cwd, &args[0]);
			if let Some(node) = g.current_computer().root.get_node(&subdir) {
				if node.is_dir() {
					g.current_computer_mut().cwd = subdir;
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

pub const CAT: Subprocess = {
	struct Cat;
	impl SubprocessFn for Cat {
		fn run(&self, g: &mut Game, args: Vec<String>) -> std::io::Result<()> {
			let cwd = g.current_computer().cwd.clone();

			let mut buf = String::new();
			for file in args {
				if let Some(node) = g.current_computer().root.get_node(&Path::parse(&cwd, &file)) {
					if let Some(f) = node.as_file() {
						buf += &f.content;
						buf.push('\n');
					}
					else {
						println!("Path \"{file}\" is not a file.");
					}
				}
				else {
					println!("File \"{file}\" does not exist.");
					return Ok(());
				}
				std::io::stdout().flush().expect("Could not flush stdio");
			}
			print!("{buf}");

			Ok(())
		}
	}
	&Cat
};