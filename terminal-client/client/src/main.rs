#[macro_use]
pub extern crate num_derive;

use bevy_reflect::Reflect;

use ipc::Connection;
use os::os_terminal;
use pinpad::pinpad_terminal;
use ratatui::layout::{Flex, Layout, Rect};
use std::{cell::RefCell, io::Result, process::ExitCode};

mod g;
mod ipc;
mod log;
mod os;
mod pinpad;
pub mod rcmut;
pub mod rl;
pub mod tui;

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
            other => Self::from(other as u8),
        }
    }
}

fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let horizontal = Layout::horizontal([width]).flex(Flex::Center);
    let vertical = Layout::vertical([height]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

fn setup() {
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
            } else {
                loop {
                    println!("Connection failed. Continue with debug? [Yn]");
                    let mut buf = String::new();
                    std::io::stdin().read_line(&mut buf)?;
                    let trimmed = buf.to_lowercase().trim().to_string();
                    if let Some(ch) = trimmed.chars().next() {
                        if ch == 'n' {
                            return Ok(GExitCode::ConnectionError);
                        } else if ch == 'y' {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                Box::new(RefCell::new(ipc::StreamConnection::io()))
            }
        };

        let message = {
            if let Ok(ipc::Message::Initialize(init)) = connection
                .borrow_mut()
                .read_message_expecting(ipc::msg::MessageType::Initialize)
            {
                init
            } else {
                return Ok(GExitCode::NoInitialization);
            }
        };

        match message.terminal_type {
            ipc::TerminalType::OS => os_terminal(connection),
            ipc::TerminalType::Pinpad => pinpad_terminal(connection),
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
        }
        Err(reason) => {
            log!("Panicked with error: {:?}", reason);
            Ok(GExitCode::Panic.into())
        }
    }
}
