use rs_ws281x::{ChannelBuilder, ControllerBuilder};
use std::time::Instant;
fn main() {
    let mut gpio_controller = ControllerBuilder::new()
        .freq(800_000)
        .dma(10)
        .channel(
            0,
            ChannelBuilder::new()
                .pin(18)
                .count(60)
                .strip_type(rs_ws281x::StripType::Ws2811Gbr)
                .brightness(255)
                .build(),
        )
        .build()
        .unwrap();

    // let start = Instant::now();
    // let mut last = 0.0;
    loop {
    //    let duration = start.elapsed().as_secs_f32();
        let leds = gpio_controller.leds_mut(0);
        for i in 60 {
            leds[i] = [255, 255, 255, 0];
        }
        gpio_controller.render().unwrap();
    }
}
