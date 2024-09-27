use std::{
    collections::HashMap,
    io::{stdout, Stdout},
    time::Instant,
};

use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{enable_raw_mode, EnterAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::{CrosstermBackend, Stylize, Terminal, *},
    style::{Color, Style},
    widgets::{
        canvas::{Canvas, Circle, Shape},
        Block, Borders, List, ListDirection, ListState,
    },
};

use sled::{driver::Driver, Sled};
use symbols::Marker;

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
    pub drivers: HashMap<Effect, Driver>,
    pub current_effect: Effect,
    last_draw: Instant,
    positions: Vec<sled::Vec2>,

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
        let positions = first_driver.positions().collect();
        App {
            should_quit: false,
            should_pause: false,
            selected_widget: SelectableWidget::Effects,
            terminal,
            drivers,
            current_effect: first_effect,
            positions,
            effects_list_state,
            last_draw: Instant::now(),
        }
    }

    pub fn heartbeat(&mut self) -> std::io::Result<()> {
        if event::poll(std::time::Duration::from_nanos(50))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    self.handle_input(key.code);
                }
            }
        }
        // terminal draws 40x a second max
        if self.last_draw.elapsed().as_nanos() > 25000000 {
            self.draw()?;
            self.last_draw = Instant::now();
        }

        if !self.should_pause {
            self.drivers.get_mut(&self.current_effect).unwrap().step();
        }

        Ok(())
    }

    pub fn draw(&mut self) -> std::io::Result<()> {
        self.terminal.draw(|frame| {
            /* variables */
            let layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Percentage(60),
                    Constraint::Percentage(20),
                ])
                .split(frame.area());

            let items = self
                .drivers
                .keys()
                .into_iter()
                .map(|effect_enum| effect_enum.as_str())
                .collect::<Vec<&str>>();

            /* effects selector */
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

            /* visualizer */

            let effect = self.current_effect.as_str();

            let running_state = if self.should_pause {
                "PAUSED"
            } else {
                "RUNNING"
            };

            let visualizer_title = format!(" {} [{}] ", effect, running_state);

            let current_driver = &self.drivers[&self.current_effect];
            let sled = current_driver.sled().unwrap();
            let domain = sled.domain();
            let center = sled.center_point();
            let x_bounds = [domain.start.x as f64, domain.end.x as f64];
            let y_bounds = [domain.start.y as f64, domain.end.y as f64];

            let canvas = Canvas::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(visualizer_title)
                        .green(),
                )
                .marker(Marker::HalfBlock)
                .x_bounds(x_bounds)
                .y_bounds(y_bounds);

            let canvas = canvas.paint(|ctx| {
                ctx.draw(&Point {
                    x: center.x,
                    y: center.y,
                    color: Color::Rgb(128, 128, 128),
                });

                for (col, pos) in current_driver.colors_and_positions_coerced::<u8>() {
                    ctx.draw(&Point {
                        x: pos.x,
                        y: pos.y,
                        color: Color::Rgb(col.red, col.green, col.blue),
                    });
                }
            });

            frame.render_widget(canvas, layout[1]);

            /* settings panel */
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

struct Point {
    pub x: f32,
    pub y: f32,
    pub color: Color,
}

impl Shape for Point {
    fn draw(&self, painter: &mut ratatui::widgets::canvas::Painter) {
        if let Some((x, y)) = painter.get_point(self.x as f64, self.y as f64) {
            painter.paint(x, y, self.color);
        }
    }
}
