// use rs_ws281x::{ChannelBuilder, Controller, ControllerBuilder};
// use sled::{color::Srgb, Sled};

mod ripples;

use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::{CrosstermBackend, Stylize, Terminal},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListDirection, ListState, Paragraph},
    Frame,
};

use std::io::{stdout, Result, Stdout};

fn draw_terminal(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    list_state: &mut ListState,
) -> Result<()> {
    terminal.draw(|frame| {
        let area = frame.size();

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(frame.size());

        let sub_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(layout[1]);

        let items = ["Comet", "Ripples", "Warpspeed"];

        let list = List::new(items)
            .block(Block::default().title("List").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().reversed())
            .highlight_symbol(" > ")
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom);

        frame.render_stateful_widget(list, sub_layout[0], list_state);

        frame.render_widget(
            Block::new()
                .borders(Borders::ALL)
                .title("Visualizer")
                .green(),
            layout[0],
        );

        frame.render_widget(
            Block::new().borders(Borders::ALL).title("Effects").yellow(),
            sub_layout[0],
        );

        frame.render_widget(
            Block::new()
                .borders(Borders::ALL)
                .title("Settings")
                .magenta(),
            sub_layout[1],
        );
    })?;

    Ok(())
}

fn main() -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;
    let mut state = ListState::default();
    state.select(Some(1));

    loop {
        draw_terminal(&mut terminal, &mut state)?;
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if key.code == KeyCode::Down {
                        state.select(Some((state.selected().unwrap() + 1) % 3));
                    } else if key.code == KeyCode::Up {
                        state.select(Some((state.selected().unwrap() - 1) % 3));
                    } else if key.code == KeyCode::Char('q') {
                        break;
                    }
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

// fn main() {
//     let sled = Sled::new("./config.toml").unwrap();
//     let num_leds = sled.num_leds();

//     let mut driver = ripples::build_driver();
//     driver.mount(sled);

//     let mut gpio_controller = construct_gpio_controller(num_leds);

//     loop {
//         driver.step();
//         let colors = driver.colors_coerced::<u8>();
//         update_gpio(&mut gpio_controller, colors);
//     }
// }

// fn construct_gpio_controller(num_leds: usize) -> Controller {
//     ControllerBuilder::new()
//         .channel(
//             0,
//             ChannelBuilder::new()
//                 .pin(18)
//                 .count(num_leds as i32)
//                 .strip_type(rs_ws281x::StripType::Ws2811Gbr)
//                 .brightness(255)
//                 .build(),
//         )
//         .build()
//         .unwrap()
// }

// fn update_gpio(controller: &mut Controller, colors: impl Iterator<Item = Srgb<u8>>) {
//     let leds = controller.leds_mut(0);

//     let mut i = 0;
//     for color in colors {
//         leds[i] = [color.red, color.green, color.blue, 0];
//         i += 1;
//     }

//     controller.render().unwrap();
// }
