use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use g::parse_command;
use ratatui::{
    prelude::{CrosstermBackend, Stylize, Terminal},
    widgets::Paragraph,
};
use std::io::{stdin, stdout, Read, Result, Write};

mod g;

fn test(){

}

fn main() -> Result<()> {
    let mut g = g::Game::default();

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

            if let Some(subprocess) = g.subprocesses.get(&proc_name.as_ref()).cloned() {
                let argv: Vec<String> = args[1..].into();
                subprocess(&mut g, argv);
            }
            else {
                println!("Unknown process \"{}\"", proc_name);
            }
        }
    }

    Ok(())
    // stdout().execute(EnterAlternateScreen)?;
    // enable_raw_mode()?;
    // let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    // terminal.clear()?;

    // loop {
    //     terminal.draw(|frame| {
    //         let area = frame.size();

    //         frame.render_widget(
    //             Paragraph::new("Hello Ratatui! (press 'q' to quit)")
    //                 .white()
    //                 .on_blue(),
    //             area,
    //         );
    //     })?;

    //     if event::poll(std::time::Duration::from_millis(16))? {
    //         if let event::Event::Key(key) = event::read()? {
    //             if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
    //                 break;
    //             }
    //         }
    //     }
    // }

    // stdout().execute(LeaveAlternateScreen)?;
    // disable_raw_mode()?;
    // Ok(())
}
