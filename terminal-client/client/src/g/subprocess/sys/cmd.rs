use rustyline::{completion::Completer, Helper, Highlighter, Hinter, Validator};

use crate::g::{
    fs::{FsError, Path},
    subprocess::{Subprocess, SubprocessFn},
    Game,
};

fn parse_command<T: IntoIterator<Item = char>>(command: T) -> Vec<String> {
    let mut in_string = false;
    let mut escaping = false;

    let mut args = Vec::<String>::new();
    let mut cur_arg = String::new();

    for ch in command.into_iter() {
        if escaping {
            cur_arg.push(ch);
            escaping = false;
        } else {
            match ch {
                '\\' => escaping = true,
                '"' => in_string = !in_string,
                ch if ch.is_whitespace() && !in_string => {
                    if !cur_arg.is_empty() {
                        args.push(cur_arg.clone());
                    }
                    cur_arg.clear();
                }
                ch => cur_arg.push(ch),
            };
        }
    }

    if !cur_arg.is_empty() {
        args.push(cur_arg);
    }

    args
}

const DEFAULT_PS1: &str = "\\u@\\H \\w$ ";

pub const CMD: Subprocess = {
    pub struct Cmd;

    #[derive(Helper, Validator, Highlighter, Hinter)]
    struct RlHelper<'a>(&'a Game);

    #[allow(dead_code)]
    impl RlHelper<'_> {
        fn find_unclosed_quote(str: &str) -> Option<(usize, char)> {
            let mut last_quote: Option<(usize, char)> = None;
            let mut escaping = false;
            for (i, ch) in str.char_indices() {
                if escaping {
                    escaping = false;
                    continue;
                }
                if let Some((_, q)) = last_quote {
                    if ch == q {
                        last_quote = None;
                    }
                    continue;
                }

                if ch == '\'' || ch == '"' {
                    last_quote = Some((i, ch));
                }
            }
            last_quote
        }

        fn find_unescaped(str: &str, search: char) -> Option<usize> {
            Self::find_unescaped_if(str, |c| c == search)
        }

        fn find_unescaped_if<P: Fn(char) -> bool>(str: &str, pred: P) -> Option<usize> {
            let mut escaping = false;
            for (i, ch) in str.char_indices() {
                if ch == '\\' {
                    escaping = true;
                } else if !escaping && pred(ch) {
                    return Some(i);
                }
            }
            None
        }

        fn rfind_unescaped_if<P: Fn(char) -> bool>(str: &str, p: P) -> Option<usize> {
            let mut candidate: Option<usize> = None;
            for (i, ch) in str.char_indices().rev() {
                if ch != '\\' && candidate.is_some() {
                    return candidate;
                } else if p(ch) {
                    candidate = Some(i)
                }
            }
            candidate
        }

        fn rfind_unescaped(str: &str, search: char) -> Option<usize> {
            Self::rfind_unescaped_if(str, |c| c == search)
        }

        fn unescape(str: &str) -> String {
            let mut buf = String::new();
            let mut quote: Option<char> = None;
            let mut escape = false;
            for ch in str.chars() {
                if escape {
                    escape = false;
                    buf.push(ch);
                } else if ch == '\\' {
                    escape = true;
                } else if let Some(q) = quote {
                    if q == ch {
                        quote = None;
                    }
                } else if ch == '"' || ch == '\'' {
                    quote = Some(ch);
                } else {
                    buf.push(ch);
                }
            }
            buf
        }

        fn escape(str: &str) -> String {
            let mut buf = String::new();
            for ch in str.chars() {
                match ch {
                    _ if ch.is_whitespace() => buf.push('\\'),
                    '\'' | '"' => buf.push('\\'),
                    _ => (),
                };
                buf.push(ch);
            }
            buf
        }
    }

    impl<'a> Completer for RlHelper<'a> {
        type Candidate = String;

        fn complete(
            &self,
            line: &str,
            pos: usize,
            _ctx: &rustyline::Context<'_>,
        ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
            // println!("COMPLETE");
            if let Some((_start_idx, _q)) = Self::find_unclosed_quote(&line[..pos]) {
                // // string is terminated mid line
                // if let Some(end_idx) = Self::find_unescaped(&line[start_idx + 1..], q) {
                // 	// let word =
                // }
                // // string goes to end of line
                // else {

                // }
                // Ok((0, vec![]))
                todo!()
            }
            // if there is an unescaped space before the cursor
            else if let Some(space_index) =
                Self::rfind_unescaped_if(&line[..pos], char::is_whitespace)
            {
                let word = if let Some(end_space) =
                    Self::find_unescaped_if(&line[pos..], char::is_whitespace)
                {
                    &line[space_index + 1..pos + end_space]
                } else {
                    &line[space_index + 1..]
                };
                let unescaped = Self::unescape(word);
                let path = Path::parse(&self.0.current_computer().cwd.borrow(), &unescaped);
                let dir_path = if unescaped.ends_with('/') {
                    path.clone()
                } else {
                    path.parent()
                };

                let basename = if unescaped.ends_with('/') {
                    String::new()
                } else {
                    path.basename().unwrap_or("".into())
                };
                if let Some(dir) = self.0.current_computer().root.get_dir(&dir_path) {
                    let subs = dir
                        .children
                        .iter()
                        .map(|c| c.data().name.clone())
                        .filter(|name| name.starts_with(&basename))
                        .map(|name| dir_path.clone().join(&Path::new(vec![name])))
                        .map(|path| Self::escape(&path.to_string()))
                        .map(|path| path + &line[space_index + 1 + unescaped.len()..])
                        .collect::<Vec<String>>();
                    Ok((space_index + 1, subs))
                } else {
                    Ok((0, vec![]))
                }
            }
            // if this is the first word
            else {
                let word = if let Some(space_idx) =
                    Self::find_unescaped_if(&line[pos..], char::is_whitespace)
                {
                    &line[..space_idx]
                } else {
                    line
                };
                let completions = self
                    .0
                    .current_computer()
                    .exes()
                    .iter()
                    .map(|exe| exe.data().name.clone())
                    .filter(|name| name.starts_with(word))
                    .collect::<Vec<String>>();

                Ok((0, completions))
            }
        }
    }

    impl SubprocessFn for Cmd {
        fn run(&self, g: &Game, _args: Vec<String>) -> std::io::Result<()> {
            let rl_helper = RlHelper(g);
            let rl_config = rustyline::Config::builder()
                .auto_add_history(true)
                .completion_type(rustyline::CompletionType::List)
                .build();

            let mut rl: rustyline::Editor<RlHelper, rustyline::history::DefaultHistory> =
                rustyline::Editor::with_config(rl_config).map_err(std::io::Error::other)?;

            rl.set_helper(Some(rl_helper));

            while !g.current_computer().should_quit.get() {
                let line = {
                    let ps1 = g
                        .current_computer()
                        .env
                        .borrow()
                        .get(&"PS1".to_string())
                        .cloned()
                        .unwrap_or(DEFAULT_PS1.into());

                    rl.readline(
                        &ps1.replace("\\u", &g.current_computer().current_user().name)
                            .replace("\\H", &g.current_computer().name)
                            .replace("\\w", &g.current_computer().cwd.borrow().to_string())
                            .to_string(),
                    )
                    .unwrap_or("".into())
                };

                if line == "exit" { 
                    break;
                }

                let args = parse_command(line.chars());

                if !args.is_empty() {
                    let proc_name = args[0].clone();

                    if let Err(FsError::NotExecutable) =
                        g.start_exe_from_path(&proc_name, &args[1..])
                    {
                        println!(
                            "Could not find process \"{}\"\nType \"help\" to list all processes.",
                            proc_name
                        );
                    }
                }
            }
            Ok(())
        }
    }
    &Cmd
};
