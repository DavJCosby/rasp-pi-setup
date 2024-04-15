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

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum Effect {
    Comet,
    Ripples,
    Warpspeed,
}

impl Effect {
    pub fn as_str(&self) -> &str {
        match self {
            Effect::Comet => "Comet",
            Effect::Ripples => "Ripples",
            Effect::Warpspeed => "Warpspeed",
        }
    }
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
    drivers: HashMap<Effect, Driver>,
    current_effect: Effect,

    /* effects widget */
    effects_list_state: ListState,
}

impl App {
    pub fn new(sled: Sled, mut drivers: HashMap<Effect, Driver>) -> Self {
        stdout().execute(EnterAlternateScreen).unwrap();
        enable_raw_mode().unwrap();
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout())).unwrap();
        terminal.clear().unwrap();

        let mut effects_list_state = ListState::default();
        effects_list_state.select(Some(0));

        let first_effect = drivers.keys().into_iter().next().unwrap().clone();
        let first_driver = drivers.get_mut(&first_effect).unwrap();
        first_driver.mount(sled);

        App {
            should_quit: false,
            should_pause: false,
            selected_widget: SelectableWidget::Effects,
            terminal,
            drivers,
            current_effect: first_effect,
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

            let items = self
                .drivers
                .keys()
                .into_iter()
                .map(|effect_enum| effect_enum.as_str())
                .collect::<Vec<&str>>();

            let mut default_style = Style::default();
            let mut highlight_style = Style::default().reversed();
            let mut highlight_symbol = "   ";
            let mut effects_title = " Effects ";
            if self.should_pause {
                default_style = Style::default().italic();
                highlight_style = Style::default().not_italic().bold();
                highlight_symbol = " > ";
                effects_title = " Effects* ";
            }

            let list = List::new(items)
                .block(Block::default().title("List").borders(Borders::ALL))
                .style(default_style)
                .highlight_style(highlight_style)
                .highlight_symbol(highlight_symbol)
                .repeat_highlight_symbol(true)
                .direction(ListDirection::TopToBottom);

            frame.render_stateful_widget(list, layout[0], &mut self.effects_list_state);

            let effects_title_style = if let SelectableWidget::Effects = self.selected_widget {
                Style::default().reversed()
            } else {
                Style::default()
            };

            frame.render_widget(
                Block::new()
                    .borders(Borders::ALL)
                    .title(effects_title)
                    .title_style(effects_title_style)
                    .yellow(),
                layout[0],
            );

            let effect = self.current_effect.as_str();

            let running_state = if self.should_pause {
                "PAUSED"
            } else {
                "RUNNING"
            };

            let visualizer_title = format!(" {} [{}] ", effect, running_state);

            frame.render_widget(
                Block::new()
                    .borders(Borders::ALL)
                    .title(visualizer_title)
                    .green(),
                layout[1],
            );

            let settings_title_style = if let SelectableWidget::Settings = self.selected_widget {
                Style::default().reversed()
            } else {
                Style::default()
            };

            frame.render_widget(
                Block::new()
                    .borders(Borders::ALL)
                    .title(" Settings ")
                    .title_style(settings_title_style)
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
                SelectableWidget::Settings => self.handle_input_settings(key_code),
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
                if let Some(e) = self.effects_list_state.selected() {
                    let old_effect = self.current_effect;

                    let effects = self.drivers.keys().into_iter().collect::<Vec<&Effect>>();
                    self.current_effect = *effects[e];

                    // handling if they hit enter on their current selection
                    if self.current_effect == old_effect {
                        if self.should_pause == false {
                            self.should_pause = true;
                            return;
                        } else {
                            self.should_pause = false;
                            return;
                        }
                    }

                    let old_driver = self.drivers.get_mut(&old_effect).unwrap();
                    let sled = old_driver.dismount();
                    let new_driver = self.drivers.get_mut(&self.current_effect).unwrap();
                    new_driver.mount(sled);

                    self.selected_widget = SelectableWidget::Settings;
                    self.should_pause = false;
                }
            }

            KeyCode::Right => self.selected_widget = SelectableWidget::Settings,

            _ => {}
        }
    }

    fn handle_input_settings(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Left => self.selected_widget = SelectableWidget::Effects,
            _ => {}
        }
    }
}