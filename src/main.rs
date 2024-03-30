use rs_ws281x::{ChannelBuilder, Controller, ControllerBuilder};
use sled::{color::Srgb, Sled};

mod comet;

fn main() {
    let sled = Sled::new("./config.toml").unwrap();
    let num_leds = sled.num_leds();
    let mut driver = comet::build_driver();
    driver.mount(sled);

    let mut gpio_controller = construct_gpio_controller(num_leds);
    //let mut scheduler = Scheduler::new(512.0);
    //scheduler.loop_forever(|| {
    loop {
        driver.step();
        let colors = driver.colors_coerced::<u8>();
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

fn update_gpio(controller: &mut Controller, colors: impl Iterator<Item = Srgb<u8>>) {
    let leds = controller.leds_mut(0);

    let mut i = 0;
    for color in colors {
        leds[i] = [color.red, color.green, color.blue, 0];
        i += 1;
    }

    controller.render().unwrap();
}
