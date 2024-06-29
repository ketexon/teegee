use std::io::Write;

use crate::g::Game;

fn parse_command<T: IntoIterator<Item = char>>(command: T) -> Vec<String> {
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
        print!("{}> ", g.c.cwd.format(&g.c.drive));
        std::io::stdout().flush()?;

        let line = {
            let mut buf = String::new();
            std::io::stdin().read_line(&mut buf)?;
            buf.trim().to_string()
        };

        let args = parse_command(line.chars());

        if args.len() > 0 {
            let proc_name = args[0].clone();

            if let Some(subprocess) = g.c.subprocesses.get(&proc_name).cloned() {
                let argv: Vec<String> = args[1..].into();
                let _ = subprocess(&mut g, argv);
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

pub fn ssh(g: &mut Game, args: Vec<String>) -> std::io::Result<()> {
	Ok(())
}