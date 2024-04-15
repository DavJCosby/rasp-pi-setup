// use rs_ws281x::{ChannelBuilder, Controller, ControllerBuilder};
// use sled::{color::Srgb, Sled};

mod effects;
mod tui;
use effects::*;

use crossterm::{
    terminal::{disable_raw_mode, LeaveAlternateScreen},
    ExecutableCommand,
};

use sled::Sled;

use std::{
    collections::HashMap,
    io::{stdout, Result},
};

fn main() -> Result<()> {
    let sled = Sled::new("./config.toml").unwrap();
    let mut drivers = HashMap::new();
    drivers.insert(tui::Effect::Comet, comet::build_driver());
    drivers.insert(tui::Effect::Ripples, ripples::build_driver());
    drivers.insert(tui::Effect::Warpspeed, warpspeed::build_driver());

    let mut app = tui::App::new(sled, drivers);

    while !app.should_quit() {
        app.heartbeat()?;
        if !app.should_pause() {}
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
