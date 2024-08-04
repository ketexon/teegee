#[macro_use]
pub extern crate num_derive;

use bevy_reflect::Reflect;

use g::fs::FsError;
use ipc::Connection;
use pinpad::terminal1;
use ratatui::layout::{Flex, Layout, Rect};
use std::{cell::RefCell, io::Result, process::ExitCode};

mod g;
mod ipc;
mod log;
mod pinpad;
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

    // this is so that, for certain tiling window managers
    // with certain term emulators
    // (caugh caugh hyprland/urxvt), they clear after 
    // the position has been set
    std::thread::sleep(std::time::Duration::from_millis(16));
    g.queue_process("clear", []);
    g.queue_process("cmd", []);
    while let Some((name, args)) = g.get_queued_process() {
        let res = g.start_exe_from_path(&name, args).map_err(|e| match e {
            FsError::NotExecutable => std::io::Error::other("Tried to run process that does not exist"),
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
                            return Ok(GExitCode::ConnectionError);
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
                return Ok(GExitCode::NoInitialization)
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
