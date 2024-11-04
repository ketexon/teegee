use std::boxed::Box;
use std::rc::Rc;
use std::{cell::RefCell, io::Result};

use crossterm::event::{
    self, DisableMouseCapture, KeyCode, KeyEventKind, MouseButton, MouseEventKind,
};
use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};
use crossterm::{
    event::EnableMouseCapture,
    terminal::{enable_raw_mode, EnterAlternateScreen},
    ExecutableCommand,
};
use layout::Position;
use ratatui::prelude::*;
use ratatui::widgets::{BorderType, Borders, Paragraph};
use ratatui::{layout::Rect, prelude::CrosstermBackend, widgets::Block, Terminal};

use crate::ipc::msg::UnlockDoorMessage;
use crate::ipc::{Message, PlaySfxMessage};
use crate::{centered_rect, ipc, GExitCode};

#[allow(clippy::boxed_local)]
pub fn pinpad_terminal(connection: Box<RefCell<dyn ipc::Connection>>) -> Result<GExitCode> {
    std::io::stdout().execute(EnterAlternateScreen)?;
    std::io::stdout().execute(EnableMouseCapture)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;

    const N_NUMS: usize = 4;

    let mut nums = [0u8; N_NUMS];
    let mut selected_num = 0;

    loop {
        let mut digit_rects: Option<Rc<[Rect]>> = None;
        terminal.draw(|f| {
            let outer_block = Block::new()
                .borders(Borders::all())
                .border_type(BorderType::Thick);

            let area = centered_rect(f.size(), 5 * 8 + 2, 5);
            let chunks = Layout::horizontal(Constraint::from_fills([1; N_NUMS]))
                .margin(1)
                .split(area);

            digit_rects = Some(chunks.clone());

            f.render_widget(outer_block, area);

            let selected_block = Block::new().bg(Color::White);

            f.render_widget(
                selected_block,
                *chunks.get(selected_num).expect("Index out of range"),
            );

            for (i, (num, chunk)) in nums.iter().zip(chunks.iter()).enumerate() {
                let paragraph_rect = Layout::vertical([
                    Constraint::Fill(1),
                    Constraint::Length(1),
                    Constraint::Fill(1),
                ])
                .split(*chunk)[1];
                let paragraph =
                    Paragraph::new(num.to_string())
                        .centered()
                        .fg(if i == selected_num {
                            Color::Black
                        } else {
                            Color::White
                        });
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
                            KeyCode::Left => {
                                selected_num =
                                    (selected_num as i32 - 1).rem_euclid(N_NUMS as i32) as usize
                            }
                            KeyCode::Right => selected_num = (selected_num + 1).rem_euclid(N_NUMS),
                            KeyCode::Up => {
                                nums[selected_num] =
                                    (nums[selected_num] as i8 + 1).rem_euclid(10i8) as u8
                            }
                            KeyCode::Down => {
                                nums[selected_num] =
                                    (nums[selected_num] as i8 - 1).rem_euclid(10i8) as u8
                            }
                            KeyCode::Char(c) if c.is_ascii_digit() => {
                                nums[selected_num] = c.to_digit(10).unwrap() as u8;
                                selected_num = std::cmp::min(selected_num + 1, N_NUMS - 1);
                            }
                            KeyCode::Backspace => {
                                nums[selected_num] = 0;
                                selected_num = std::cmp::max(selected_num as i32 - 1, 0) as usize;
                            }
                            _ => {
                                play_sfx = false;
                            }
                        }

                        if play_sfx {
                            connection
                                .borrow_mut()
                                .write_message(Message::PlaySfx(PlaySfxMessage { id: 0 }))?;
                        }
                    }
                }
                event::Event::Mouse(mouse) => {
                    if let MouseEventKind::Down(mouse_button) = mouse.kind {
                        let pos = Position::new(mouse.column, mouse.row);

                        if let Some(digit_rects) = digit_rects {
                            for (i, rect) in (*digit_rects).iter().enumerate() {
                                if rect.contains(pos) {
                                    if selected_num == i {
                                        if mouse_button == MouseButton::Left {
                                            nums[i] = (nums[selected_num] as i8 + 1)
                                                .rem_euclid(10i8)
                                                as u8
                                        } else if mouse_button == MouseButton::Right {
                                            nums[i] = (nums[selected_num] as i8 - 1)
                                                .rem_euclid(10i8)
                                                as u8
                                        }
                                    } else {
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

    std::io::stdout().execute(DisableMouseCapture)?;
    std::io::stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    connection
        .borrow_mut()
        .write_message(Message::UnlockDoor(UnlockDoorMessage { code: nums }))
        .and(Ok(GExitCode::Success))
}
