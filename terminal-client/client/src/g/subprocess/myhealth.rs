use crossterm::{event::{self, KeyCode, KeyEventKind}, terminal::{self, EnterAlternateScreen, LeaveAlternateScreen}, ExecutableCommand};
use ratatui::{layout::{Constraint, Layout}, prelude::CrosstermBackend, style::{Color, Modifier, Style, Stylize}, text::{Line, Span}, widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Tabs, Widget}};
use tui_big_text::{BigText, PixelSize};

use crate::g::subprocess::SubprocessFn;

use super::Subprocess;

struct MyHealthRecord {
    pub date: &'static str,
    pub title: &'static str,
    pub notes: &'static str,
}

struct MyHealthState {
    pub records: Vec<MyHealthRecord>,
    pub list_state: ListState,
    pub selecting_tabs: bool,
    pub selected_tab: usize,
}

struct Home;

impl Widget for Home {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let layout = Layout::vertical([
            Constraint::Fill(1), 
            Constraint::Length(2),
            Constraint::Length(5),
            Constraint::Length(1),
            Constraint::Fill(1), 
        ]).split(area);

        Block::new()
            .borders(Borders::all())
            .border_style(Style::default().fg(Color::White))
            .render(area, buf);

        Paragraph::new(vec![
            Span::styled("welcome to", Style::new().add_modifier(Modifier::ITALIC)).into(),
        ])
            .centered()
            .render(layout[1], buf);

        let big_text = BigText::builder()
            .pixel_size(PixelSize::Quadrant)
            .style(Style::new().white())
            .lines(vec![
                "MyHealth".into()
            ])
            .centered()
            .build();

        big_text.render(layout[2], buf);

        Paragraph::new(vec![
            Span::styled("No new notifications", Style::new().add_modifier(Modifier::ITALIC)).into(),
        ])
            .centered()
            .render(layout[3], buf);
    }
}

struct HealthHistory;
impl Widget for HealthHistory {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        // allergies
        // medication
        // medical conditions
        // family history
        // hospitalizations
        // surgeries/procedures
        let text_style = Style::new().white();
        Paragraph::new(vec![
            Line::styled("Allergies", text_style).bold(),
            Line::styled("No known allergies", text_style),
            Line::default(),
            Line::styled("Medications", text_style),
        ])
            .block(Block::new().borders(Borders::all()).border_style(Style::new().white()))
            .render(area, buf)
    }
}

struct Records;
impl StatefulWidget for Records {
    type State = MyHealthState;
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State) {
        let layout = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Fill(2),
        ]).margin(1).spacing(1).split(area);

        let list = List::new(
            state.records.iter()
                .map(|r| ListItem::new(
                    vec![
                        Line::styled(r.date, Style::default()).add_modifier(Modifier::ITALIC),
                        Line::styled(r.title, Style::default()).add_modifier(Modifier::BOLD),
                    ]
                ))
                .collect::<Vec<ListItem>>()
        )
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Black).bg(Color::White));

        Block::new()
            .borders(Borders::all())
            .border_style(Style::default().fg(Color::White))
            .render(area, buf);

        StatefulWidget::render(
            list, 
            layout[0],
            buf,
            &mut state.list_state
        );

        if let Some(record_idx) = state.list_state.selected() {
            let record = &state.records[record_idx];
            Paragraph::new(record.notes)
                .render(layout[1], buf);
        }
    }
}

pub const MYHEALTH: Subprocess = {
    struct MyHealth;

    impl SubprocessFn for MyHealth {
        fn run(&self, g: &crate::g::Game, args: Vec<String>) -> std::io::Result<()> {
            let mut state = MyHealthState {
                records: vec![
                    MyHealthRecord { 
                        date: "01/20/93",
                        title: "Office visit",
                        notes: "Notes 1",
                    },
                    MyHealthRecord { 
                        date: "05/03/93",
                        title: "Inpatient",
                        notes: "Notes 2",
                    },
                ],
                list_state: ListState::default(),
                selecting_tabs: true,
                selected_tab: 0,
            };

            std::io::stdout().execute(EnterAlternateScreen)?;
            terminal::enable_raw_mode()?;

            let mut term = ratatui::Terminal::new(CrosstermBackend::new(std::io::stdout()))?;

            let tabs = vec![
                "Home",
                "Health History",
                "Records",
            ];
            const HOME_INDEX: usize = 0;
            const HISTORY_INDEX: usize = 1;
            const RECORDS_INDEX: usize = 2;
            loop {
                term.draw(|frame| {
                    let layout = Layout::vertical(vec![
                        Constraint::Length(1),
                        Constraint::Fill(1),
                    ]).split(frame.size());
                    let tabs_widget = Tabs::new(tabs.clone())
                        .style(Style::default().fg(Color::White))
                        .highlight_style(Style::default().fg(Color::Black).bg(Color::White))
                        .select(state.selected_tab);

                    let tabs_area = layout[0];
                    let body_area = layout[1];

                    frame.render_widget(
                        tabs_widget,
                        tabs_area
                    );
                    match state.selected_tab {
                        HOME_INDEX => frame.render_widget(Home, body_area),
                        HISTORY_INDEX => frame.render_widget(HealthHistory, body_area),
                        RECORDS_INDEX => frame.render_stateful_widget(Records, body_area, &mut state),
                        _ => panic!("Unknown"),
                    };
                })?;

                if event::poll(std::time::Duration::from_millis(16))? {
                   if let event::Event::Key(k) = event::read()? {
                        if k.kind == KeyEventKind::Press {
                            match k.code {
                                KeyCode::Char('q') => { break; },
                                KeyCode::Esc if state.selecting_tabs => { break; },
                                KeyCode::Left | KeyCode::Char('h') if state.selecting_tabs => { 
                                    state.selected_tab = (state.selected_tab as i32 - 1).rem_euclid(tabs.len() as i32) as usize;
                                },
                                KeyCode::Right | KeyCode::Char('l') if state.selecting_tabs => {
                                    state.selected_tab = (state.selected_tab as i32 + 1).rem_euclid(tabs.len() as i32) as usize;
                                },
                                _ => ()
                            }
                            if state.selected_tab == RECORDS_INDEX {
                                match k.code {
                                    KeyCode::Enter if state.selecting_tabs => {
                                        state.selecting_tabs = false;
                                        state.list_state.select(Some(0));
                                    },
                                    KeyCode::Esc if !state.selecting_tabs => {
                                        state.selecting_tabs = true;
                                        state.list_state.select(None);
                                    },
                                    KeyCode::Up | KeyCode::Char('k') if !state.selecting_tabs => {
                                        state.list_state.select_previous();
                                    },
                                    KeyCode::Down | KeyCode::Char('j') if !state.selecting_tabs => {
                                        state.list_state.select_next();
                                    },
                                    _ => ()
                                }
                            }
                        }
                    }
                }
            }

            terminal::disable_raw_mode()?;
            std::io::stdout().execute(LeaveAlternateScreen)?;

            Ok(())
        }
    }

    &MyHealth
};

pub const DEFAULT: &[(&str, Subprocess)] = &[
    ("myhealth", MYHEALTH),
];
