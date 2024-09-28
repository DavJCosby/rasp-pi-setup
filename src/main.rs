use std::time::Instant;

use rpi_ws281x_c::{Channel, Driver, DriverBuilder, SpiPin};

//use rs_ws281x::{ChannelBuilder, Controller, ControllerBuilder};
use sled::{color::Srgb, Sled};

mod effects;
// mod tui;
use effects::*;

// use crossterm::{
//     terminal::{disable_raw_mode, LeaveAlternateScreen},
//     ExecutableCommand,
// };

// use std::{
//     collections::HashMap,
//     io::{stdout, Result},
// };

// fn main() -> Result<()> {
//     let sled = Sled::new("./config.toml").unwrap();
//     let mut drivers = HashMap::new();
//     drivers.insert(tui::Effect::Comet, comet::build_driver());
//     drivers.insert(tui::Effect::Ripples, ripples::build_driver());
//     drivers.insert(tui::Effect::Warpspeed, warpspeed::build_driver());

//     let mut app = tui::App::new(sled, drivers);
//     let mut gpio_controller = construct_gpio_controller(400);

//     while !app.should_quit() {
//         app.heartbeat()?;
//         if !app.should_pause() {
//             let colors = app.drivers.get(&app.current_effect).unwrap().colors_coerced::<u8>();
//             update_gpio(&mut gpio_controller, colors);
//         }
//     }

//     stdout().execute(LeaveAlternateScreen)?;
//     disable_raw_mode()?;
//     Ok(())
// }

fn main() {
    let sled = Sled::new("./config.yap").unwrap();
    let num_leds = sled.num_leds();
    println!("Starting SLED system of {} LEDs.", num_leds);

    let mut driver = ripples::build_driver();
    driver.mount(sled);

    let mut gpio_controller = construct_gpio_controller(num_leds);
    let mut last_printout = Instant::now();
    let mut updates = 0;
    loop {
        updates += 1;
        if last_printout.elapsed().as_secs_f32() > 2.0 {
            let hz = (updates as f32) / 2.0;
            println!("Running at {} Hz.", hz);
            updates = 0;
            last_printout = Instant::now();
        }
        driver.step();
        let colors = driver.colors();
        update_gpio(&mut gpio_controller, colors);
    }
}

fn construct_gpio_controller(num_leds: usize) -> Driver {
    DriverBuilder::new()
        .channel1(
            Channel::set_spi(SpiPin::Spi0)
                .set_led_count(num_leds as u16)
                .set_strip_gbr(),
        )
        .build()
        .unwrap()

    // ControllerBuilder::new()

    //     .channel(
    //         0,
    //         ChannelBuilder::new()
    //             .pin(18)
    //             .count(num_leds as i32)
    //             .strip_type(rs_ws281x::StripType::Ws2811Gbr)
    //             .brightness(255)
    //             .build(),
    //     )
    //     .build()
    //     .unwrap()
}

fn update_gpio<'a>(controller: &mut Driver, colors: impl Iterator<Item = &'a Srgb>) {
    let channel = controller.channel1_mut();
    let leds = channel.leds_mut();

    for led in leds {
        *led = std::u32::MAX; // white?
    }

    controller.render().unwrap();
    // let leds = controller.leds_mut(0);

    // let mut i = 0;
    // for color in colors {
    //     leds[i] = [
    //         (color.red * 255.0) as u8,
    //         (color.green * 255.0) as u8,
    //         (color.blue * 255.0) as u8,
    //         0,
    //     ];
    //     i += 1;
    // }

    // controller.render().unwrap();
}
