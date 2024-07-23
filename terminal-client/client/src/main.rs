#[macro_use]
pub extern crate num_derive;

use bevy_reflect::Reflect;
use crossterm::{event::{self, DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEventKind, MouseButton, MouseEventKind}, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}, ExecutableCommand};

use g::fs::FsError;
use ipc::{msg::{Message, UnlockDoorMessage}, Connection, PlaySfxMessage};
use ratatui::{backend::CrosstermBackend, layout::{Constraint, Flex, Layout, Position, Rect}, style::{Color, Stylize}, widgets::{Block, BorderType, Borders, Paragraph}, Terminal};
use std::{cell::RefCell, io::{stdout, Result}, process::ExitCode, rc::Rc};

mod g;
mod ipc;
mod log;
pub mod rcmut;
pub mod rl;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
enum GExitCode {
    Success = 0,
    Failure = 1,
    ConnectionError = 2,
    NoInitialization = 3,
    Panic = 4,
}

impl From<GExitCode> for ExitCode {
    fn from(value: GExitCode) -> Self {
        match value {
            GExitCode::Success => ExitCode::SUCCESS,
            GExitCode::Failure => ExitCode::FAILURE,
            other => Self::from(other as u8)
        }
    }
}

fn terminal0(connection: Box<RefCell<dyn ipc::Connection>>) -> Result<GExitCode> {
    let g = g::Game::new(connection);

    g.queue_process("cmd", []);
    while let Some((name, args)) = g.get_queued_process() {
        let res = g.start_exe_from_path(&name, args).map_err(|e| match e {
            FsError::PathIsNotExecutable => std::io::Error::other("Tried to run process that does not exist"),
            e => std::io::Error::other(format!("Unknown error: {e:?}")),
        });

        if let Err(e) = res {
            log!("Error in terminal0: {e:?}");
        }
    }

    Ok(GExitCode::Success)
}

fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let horizontal = Layout::horizontal([width]).flex(Flex::Center);
    let vertical = Layout::vertical([height]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

fn terminal1(connection: Box<RefCell<dyn ipc::Connection>>) -> Result<GExitCode> {
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(EnableMouseCapture)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    const N_NUMS: usize = 4;

    let mut nums = [0u8;N_NUMS];
    let mut selected_num = 0;

    loop {
        let mut digit_rects: Option<Rc<[Rect]>> = None;
        terminal.draw(|f| {
            let outer_block = Block::new()
                .borders(Borders::all())
                .border_type(BorderType::Thick);

            let area = centered_rect(f.size(), 5 * 8+2, 5);
            let chunks = Layout::horizontal(Constraint::from_fills([1;N_NUMS]))
                .margin(1)
                .split(area);

            digit_rects = Some(chunks.clone());

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
            match event::read()? {
                event::Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        let mut play_sfx = true;
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
                            _ => { play_sfx = false; }
                        }

                        if play_sfx {
                            connection.borrow_mut().write_message(Message::PlaySfx(PlaySfxMessage {
                                id: 0,
                            }))?;
                        }
                    }
                },
                event::Event::Mouse(mouse) => {
                    if let MouseEventKind::Down(mouse_button) = mouse.kind {
                        let pos = Position::new(mouse.column, mouse.row);

                        println!("CLICK");
                        if let Some(digit_rects) = digit_rects {
                            for (i, rect) in (&*digit_rects).iter().enumerate() {
                                if rect.contains(pos) {
                                    if selected_num == i {
                                        if mouse_button == MouseButton::Left {
                                            nums[i] = (nums[selected_num] as i8 + 1).rem_euclid(10i8) as u8
                                        } else if mouse_button == MouseButton::Right {
                                            nums[i] = (nums[selected_num] as i8 - 1).rem_euclid(10i8) as u8
                                        }
                                    }
                                    else {
                                        selected_num = i;
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
                _ => (),
            }
        }
    }

    stdout().execute(DisableMouseCapture)?;
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    connection.borrow_mut().write_message(Message::UnlockDoor(UnlockDoorMessage {
        code: nums
    })).and(Ok(GExitCode::Success))
}

fn setup(){
    log::init();
}

#[derive(Reflect)]
struct Test {
    pub x: i32,
}

fn main() -> Result<ExitCode> {
    setup();

    fn main_impl() -> Result<GExitCode> {
        let connection: Box<RefCell<dyn Connection>> = {
            if let Some(c) = ipc::StreamConnection::tcp() {
                Box::new(RefCell::new(c))
            }
            else {
                loop {
                    println!("Connection failed. Continue with debug? [Yn]");
                    let mut buf = String::new();
                    std::io::stdin().read_line(&mut buf)?;
                    let trimmed = buf.to_lowercase().trim().to_string();
                    if let Some(ch) = trimmed.chars().next() {
                        if ch == 'n' {
                            return Ok(GExitCode::ConnectionError.into());
                        }
                        else if ch == 'y' {
                            break;
                        }
                    }
                    else {
                        break;
                    }
                }
                Box::new(RefCell::new(ipc::StreamConnection::io()))
            }
        };

        let message = {
            if let Ok(ipc::Message::Initialize(init)) = connection.borrow_mut().read_message_expecting(ipc::msg::MessageType::Initialize) {
                init
            }
            else {
                return Ok(GExitCode::NoInitialization.into())
            }
        };

        match message.terminal_type {
            ipc::TerminalType::OS => terminal0(connection),
            ipc::TerminalType::Pinpad => terminal1(connection),
        }
    }

    let res = std::panic::catch_unwind(main_impl);

    match res {
        Ok(res) => {
            match &res {
                Ok(GExitCode::Success) => (),
                Ok(exit_code) => {
                    log!("Unsuccessful exit: {:?}", exit_code);
                }
                Err(e) => {
                    log!("IO Error: {:?}", e);
                }
            }
            res.map(|exit_code| exit_code.into())
        },
        Err(reason) => {
            log!("Panicked with error: {:?}", reason);
            Ok(GExitCode::Panic.into())
        }
    }
}