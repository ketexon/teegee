use std::io::Write;

use crate::g::{fs, Game};

pub fn ls(g: &mut Game, args: Vec<String>) -> std::io::Result<()>{
	if let Some(entry) = g.c.root.get_node(&g.c.cwd) {
		if let fs::NodeContent::Dir(dir) = &entry.content {
			let mut v: Vec<&fs::Node> = dir.children.iter().collect();
			v.sort_unstable_by(|a, b| match (&a.content, &b.content) {
				(fs::NodeContent::Dir(_), fs::NodeContent::Dir(_))
				| (fs::NodeContent::File(_), fs::NodeContent::File(_)) => a.name.cmp(&b.name),
				(fs::NodeContent::Dir(_), fs::NodeContent::File(_)) => std::cmp::Ordering::Less,
				(fs::NodeContent::File(_), fs::NodeContent::Dir(_)) => std::cmp::Ordering::Greater,
			});

			let rows = v.iter().map(|node| [
				format!(
					"{}{}{}",
					if node.is_dir() { 'd' } else { '-' },
					node.owner_security,
					node.other_security
				),
				format!("{}", node.date),
				if let fs::NodeContent::File(f) = &node.content {
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

pub fn cd(g: &mut Game, args: Vec<String>) -> std::io::Result<()> {
	if args.len() != 1 {
		println!("Expected 1 argument.");
		return Ok(());
	}
	let subdir = fs::Path::parse(&g.c.cwd, args[0].clone());
	if let Some(fs::Node { content: fs::NodeContent::Dir(_), .. }) = g.c.root.get_node(&subdir) {
		g.c.cwd = subdir;
	}
	else {
		println!("No such directory \"{}\".", subdir.format(&g.c.drive));
	}
	Ok(())
}

pub fn cat(g: &mut Game, args: Vec<String>) -> std::io::Result<()> {
	if let Some(fs::Node { content: fs::NodeContent::Dir(d), .. }) = g.c.root.get_node(&g.c.cwd) {
		let mut buf = String::new();
		for file in args {
			if let Some(fs::Node { content: fs::NodeContent::File(f), .. }) = d.get_child(&file) {
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
