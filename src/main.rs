use rs_ws281x::{ChannelBuilder, Controller, ControllerBuilder};
use sled::{
    color::Rgb,
    driver::{BufferContainer, Driver, Filters, TimeInfo},
    scheduler::Scheduler,
    Sled, SledError,
};

const NUM_LEDS: i32 = 60;

fn main() {
    let sled = Sled::new("./config.toml").unwrap();
    let mut driver = Driver::new();
    driver.set_draw_commands(draw);
    driver.mount(sled);

    let mut gpio_controller = construct_gpio_controller();
    let mut scheduler = Scheduler::fixed_hz(500.0);
    scheduler.loop_until_err(|| {
        driver.step();
        let colors: Vec<Rgb<_, u8>> = driver.read_colors();
        update_gpio(&mut gpio_controller, &colors);
        Ok(())
    });
}

fn draw(
    sled: &mut Sled,
    _buffers: &BufferContainer,
    _filters: &Filters,
    time_info: &TimeInfo,
) -> Result<(), SledError> {
    sled.map(|led| led.color * 0.85);
    let _ = sled.set_at_angle(time_info.elapsed.as_secs_f32(), Rgb::new(1.0, 1.0, 1.0));
    Ok(())
}

fn construct_gpio_controller() -> Controller {
    ControllerBuilder::new()
        .freq(800_000)
        .dma(10)
        .channel(
            0,
            ChannelBuilder::new()
                .pin(18)
                .count(NUM_LEDS)
                .strip_type(rs_ws281x::StripType::Ws2811Gbr)
                .brightness(255)
                .build(),
        )
        .build()
        .unwrap()
}

fn update_gpio<T>(controller: &mut Controller, colors: &Vec<Rgb<T, u8>>) {
    let leds = controller.leds_mut(0);
    for i in 0..NUM_LEDS {
        let (r, g, b) = colors[i as usize].into_components();
        leds[i as usize] = [r, g, b, 0];
    }
    controller.render().unwrap();
}
