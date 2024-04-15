use std::{
    collections::HashMap,
    io::{stdout, Stdout},
};

use crossterm::{
    event::{self, KeyCode, KeyEvent, KeyEventKind},
    terminal::{enable_raw_mode, EnterAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::*,
    prelude::{CrosstermBackend, Stylize, Terminal},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListDirection, ListState, Paragraph},
    Frame,
};

use sled::{driver::Driver, Sled};

enum SelectableWidget {
    Effects,
    Settings,
}

#[derive(PartialEq, Eq, Hash)]
pub enum Effects {
    Comet,
    Ripples,
    Warpspeed,
}

impl Default for SelectableWidget {
    fn default() -> Self {
        SelectableWidget::Effects
    }
}

pub struct App {
    should_quit: bool,
    should_pause: bool,
    selected_widget: SelectableWidget,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    sled: Sled,
    drivers: HashMap<Effects, Driver>,
    current_effect: Option<Effects>,

    /* effects widget */
    effects_list_state: ListState,
}

impl App {
    pub fn new(sled: Sled, drivers: HashMap<Effects, Driver>) -> Self {
        stdout().execute(EnterAlternateScreen).unwrap();
        enable_raw_mode().unwrap();
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout())).unwrap();
        terminal.clear().unwrap();

        let mut effects_list_state = ListState::default();
        effects_list_state.select(Some(0));

        App {
            should_quit: false,
            should_pause: false,
            selected_widget: SelectableWidget::Effects,
            terminal,
            sled,
            drivers,
            current_effect: None,
            effects_list_state,
        }
    }

    pub fn heartbeat(&mut self) -> std::io::Result<()> {
        if event::poll(std::time::Duration::from_millis(1))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    self.handle_input(key.code);
                }
            }
        }

        self.draw()?;

        Ok(())
    }

    pub fn draw(&mut self) -> std::io::Result<()> {
        self.terminal.draw(|frame| {
            let layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(25),
                    Constraint::Percentage(60),
                    Constraint::Percentage(25),
                ])
                .split(frame.size());

            let items = ["Comet", "Ripples", "Warpspeed"];

            let list = List::new(items)
                .block(Block::default().title("List").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().reversed())
                .highlight_symbol(" > ")
                .repeat_highlight_symbol(true)
                .direction(ListDirection::TopToBottom);

            frame.render_stateful_widget(list, layout[0], &mut self.effects_list_state);

            let visualizer_title = match self.should_pause {
                true => "Visualizer [PAUSED]",
                false => "Visualizer [RUNNING]",
            };

            frame.render_widget(
                Block::new()
                    .borders(Borders::ALL)
                    .title(visualizer_title)
                    .green(),
                layout[1],
            );

            frame.render_widget(
                Block::new().borders(Borders::ALL).title("Effects").yellow(),
                layout[0],
            );

            frame.render_widget(
                Block::new()
                    .borders(Borders::ALL)
                    .title("Settings")
                    .magenta(),
                layout[2],
            );
        })?;

        Ok(())
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn should_pause(&self) -> bool {
        self.should_pause
    }

    fn handle_input(&mut self, key_code: KeyCode) {
        if key_code == event::KeyCode::Char('q') {
            self.should_quit = true;
            return;
        } else {
            match self.selected_widget {
                SelectableWidget::Effects => self.handle_input_effects(key_code),
                SelectableWidget::Settings => todo!(),
            }
        }
    }

    fn handle_input_effects(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Down => {
                self.should_pause = true;
                self.effects_list_state.select(Some(
                    (self.effects_list_state.selected().unwrap() + 1) % self.drivers.len(),
                ))
            }
            KeyCode::Up => {
                self.should_pause = true;
                self.effects_list_state.select(Some(
                    (self.effects_list_state.selected().unwrap() - 1) % self.drivers.len(),
                ))
            }
            KeyCode::Enter => {
                self.should_pause = false;
            }

            _ => {}
        }
    }
}
