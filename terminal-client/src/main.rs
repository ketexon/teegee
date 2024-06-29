#[macro_use]
pub extern crate num_derive;

use crossterm::{event::{self, KeyCode, KeyEventKind}, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}, ExecutableCommand};

use ipc::msg::{Message, UnlockDoorMessage};
use num_traits::{FromPrimitive, ToPrimitive};
use ratatui::{backend::CrosstermBackend, layout::{Constraint, Flex, Layout, Rect}, style::{Color, Stylize}, widgets::{Block, BorderType, Borders, Paragraph}, Terminal};
use std::{fmt::{Debug, Display}, io::{stdout, Result}, process::ExitCode};

mod g;
mod ipc;

#[derive(Clone, Copy)]
enum GExitCode {
    Success = 0,
    Failure = 1,
    ConnectionError = 2,
    NoInitialization = 3,
    CouldNotWritePipe = 4,
}

impl From<GExitCode> for ExitCode {
    fn from(value: GExitCode) -> Self {
        Self::from(value as u8)
    }
}

fn terminal0() -> Result<ExitCode> {
    let mut g = g::Game::default();

    g.start_process("cmd", [])?;

    Ok(ExitCode::SUCCESS)
}

fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let horizontal = Layout::horizontal([width]).flex(Flex::Center);
    let vertical = Layout::vertical([height]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

fn report_delta<T: Display>(last: &mut std::time::Instant, label: T){
    std::println!("{}: {:.2?}", label, last.elapsed());
    *last = std::time::Instant::now();
}

fn terminal1(connection: &mut ipc::Connection) -> Result<ExitCode> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    const N_NUMS: usize = 4;

    let mut nums = [0u8;N_NUMS];
    let mut selected_num = 0;

    loop {
        terminal.draw(|f| {
            let outer_block = Block::new()
                .borders(Borders::all())
                .border_type(BorderType::Thick);

            let area = centered_rect(f.size(), 5 * 8+2, 5);
            let chunks = Layout::horizontal(Constraint::from_fills([1;N_NUMS]))
                .margin(1)
                .split(area);

            f.render_widget(outer_block, area);

            let selected_block = Block::new()
                .bg(Color::White);

            f.render_widget(selected_block, *chunks.get(selected_num).expect("Index out of range"));

            for (i, (num, chunk)) in nums.iter().zip(chunks.iter()).enumerate() {

                let paragraph_rect = Layout::vertical([
                    Constraint::Fill(1),
                    Constraint::Length(1),
                    Constraint::Fill(1),
                ]).split(*chunk)[1];
                let paragraph = Paragraph::new(num.to_string())
                    .centered()
                    .fg(if i == selected_num { Color::Black } else { Color::White });
                f.render_widget(paragraph, paragraph_rect);
            }
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Enter => break,
                        KeyCode::Left => selected_num = (selected_num as i32 - 1).rem_euclid(N_NUMS as i32) as usize,
                        KeyCode::Right => selected_num = (selected_num + 1).rem_euclid(N_NUMS),
                        KeyCode::Up => nums[selected_num] = (nums[selected_num] as i8 + 1).rem_euclid(10i8) as u8,
                        KeyCode::Down => nums[selected_num] = (nums[selected_num] as i8 - 1).rem_euclid(10i8) as u8,
                        KeyCode::Char(c) if c.is_digit(10) => {
                            nums[selected_num] = c.to_digit(10).unwrap() as u8;
                            selected_num = std::cmp::min(selected_num + 1, N_NUMS - 1);
                        },
                        KeyCode::Backspace => {
                            nums[selected_num] = 0;
                            selected_num = std::cmp::max(selected_num as i32 - 1, 0) as usize;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    connection.write_message(Message::UnlockDoor(UnlockDoorMessage {
        code: nums
    })).and(Ok(ExitCode::SUCCESS))
}

fn main() -> Result<ExitCode> {
    let mut connection = {
        if let Some(c) = ipc::Connection::new() {
            c
        }
        else {
            return Ok(GExitCode::ConnectionError.into())
        }
    };

    let message = {
        if let Ok(ipc::Message::Initialize(init)) = connection.read_message() {
            init
        }
        else {
            return Ok(GExitCode::NoInitialization.into())
        }
    };

    match message.terminal_type {
        ipc::TerminalType::OS => terminal0(),
        ipc::TerminalType::Pinpad => terminal1(&mut connection),
    }
}