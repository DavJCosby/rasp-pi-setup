use rs_ws281x::{ChannelBuilder, Controller, ControllerBuilder};
use sled::{color::Srgb, Sled};

mod warpspeed;

fn main() {
    let sled = Sled::new("./config.toml").unwrap();
    let num_leds = sled.num_leds();
    let mut driver = warpspeed::build_driver();
    driver.mount(sled);

    let mut gpio_controller = construct_gpio_controller(num_leds);
    //let mut scheduler = Scheduler::new(512.0);
    //scheduler.loop_forever(|| {
    loop {
        driver.step();
        let colors = driver.colors();
        update_gpio(&mut gpio_controller, colors);
    }
    //})
}

fn construct_gpio_controller(num_leds: usize) -> Controller {
    ControllerBuilder::new()
        .freq(800_000)
        .dma(10)
        .channel(
            0,
            ChannelBuilder::new()
                .pin(18)
                .count(num_leds as i32)
                .strip_type(rs_ws281x::StripType::Ws2811Gbr)
                .brightness(255)
                .build(),
        )
        .build()
        .unwrap()
}

fn update_gpio(controller: &mut Controller, colors: impl Iterator<Item = Srgb<f32>>) {
    let leds = controller.leds_mut(0);

    let mut i = 0;
    for color in colors {
        leds[i] = [(color.red / (1.0 + color.red)) as u8, (color.green / (1.0 + color.green)) as u8, (color.blue / (1.0 + color.blue)) as u8, 0];
        i += 1;
    }

    controller.render().unwrap();
}
