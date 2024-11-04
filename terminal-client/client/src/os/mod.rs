use std::{cell::RefCell, io::Result};

use crate::g::computer::ComputerId;
use crate::{
    g::{self, fs::FsError},
    ipc, log, GExitCode,
};

pub fn os_terminal(connection: Box<RefCell<dyn ipc::Connection>>) -> Result<GExitCode> {
    let initial_computer = match connection.borrow_mut().read_message() {
        Ok(msg) => match msg {
            ipc::Message::InitializeOS(msg) => msg.computer_id,
            other => {
                log!("Expected InitializeOS, got {other:?}");
                return Ok(GExitCode::NoInitialization);
            }
        },
        Err(e) => {
            log!("Could not read InitializeOS: {e:?}");
            return Ok(GExitCode::NoInitialization);
        }
    };

    log!("Successfully initialized OS. Initial Computer: {initial_computer:?}.");

    let g = g::Game::new(connection, initial_computer);

    // this is so that, for certain tiling window managers
    // with certain term emulators
    // (caugh caugh hyprland/urxvt), they clear after
    // the position has been set
    std::thread::sleep(std::time::Duration::from_millis(16));
    g.queue_process("clear", []);
    g.queue_process("cmd", []);
    while let Some((name, args)) = g.get_queued_process() {
        let res = g.start_exe_from_path(&name, args).map_err(|e| match e {
            FsError::NotExecutable => {
                std::io::Error::other("Tried to run process that does not exist")
            }
            e => std::io::Error::other(format!("Unknown error: {e:?}")),
        });

        if let Err(e) = res {
            log!("Error in os_terminal: {e:?}");
        }
    }

    Ok(GExitCode::Success)
}
