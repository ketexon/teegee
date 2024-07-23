use crate::g::{fs::Path, subprocess::{Subprocess, SubprocessFn}, Game};

pub const LS: Subprocess = {
	struct Ls;
	impl SubprocessFn for Ls {
		fn run(&self, g: &Game, args: Vec<String>) -> std::io::Result<()> {
			if args.len() > 1 {
				let _ = g.start_exe_from_path("help", vec!["ls".into()]);
				return Ok(());
			}

			let cwd = g.current_computer().cwd.borrow().clone();
			let dir = if args.len() == 1 {
				Path::parse(&cwd, args.get(0).unwrap())
			} else {
				cwd
			};

			if let Some(entry) = g.current_computer().root.get_node(&dir) {
				if let Some(dir) = &entry.as_dir() {
					use crate::g::fs;
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
			else {
				println!("\"{dir}\" is not a directory");
			}
			Ok(())
		}
	}

	&Ls
};
