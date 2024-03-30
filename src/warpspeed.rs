use rand::rngs::ThreadRng;
use rand::Rng;
use sled::driver::{BufferContainer, Driver, Filters, TimeInfo};
use sled::{color::Rgb, Sled, SledError, Vec2};

const NUM_STARS: usize = 5000;
const VELOCITY: f32 = 3.0;

#[allow(dead_code)]
pub fn build_driver() -> Driver {
    let mut driver = Driver::new();

    driver.set_startup_commands(startup);
    driver.set_compute_commands(compute);
    driver.set_draw_commands(draw);

    return driver;
}

fn startup(
    sled: &mut Sled,
    buffers: &mut BufferContainer,
    _filters: &mut Filters,
) -> Result<(), SledError> {
    let stars = buffers.create_buffer::<Vec2>("stars");
    let center = sled.center_point();
    let mut rng = rand::thread_rng();

    for _ in 0..NUM_STARS {
        stars.push(random_spawn_location(&mut rng, center));
    }

    let colors = buffers.create_buffer::<Rgb>("colors");
    colors.extend([
        Rgb::new(0.15, 0.5, 1.0),
        Rgb::new(0.25, 0.3, 1.0),
        Rgb::new(0.05, 0.4, 0.8),
        Rgb::new(0.7, 0.0, 0.6),
        Rgb::new(0.05, 0.75, 1.0),
        Rgb::new(0.1, 0.8, 0.6),
        Rgb::new(0.6, 0.05, 0.2),
        Rgb::new(0.85, 0.15, 0.3),
        Rgb::new(0.0, 0.0, 1.0),
        Rgb::new(1.0, 0.71, 0.705),
    ]);

    Ok(())
}

fn compute(
    sled: &Sled,
    buffers: &mut BufferContainer,
    _filters: &mut Filters,
    time_info: &TimeInfo,
) -> Result<(), SledError> {
    let mut rng = rand::thread_rng();
    let delta = time_info.delta.as_secs_f32();
    let stars = buffers.get_buffer_mut::<Vec2>("stars")?;
    let center = sled.center_point();

    for star in stars {
        star.y -= VELOCITY * delta;
        if star.y < -50.0 {
            *star = random_spawn_location(&mut rng, center)
        }
    }

    Ok(())
}

fn draw(
    sled: &mut Sled,
    buffers: &BufferContainer,
    _filters: &Filters,
    _time_info: &TimeInfo,
) -> Result<(), SledError> {
    let stars = buffers.get_buffer::<Vec2>("stars")?;
    let center = sled.center_point();

    sled.for_each(|led| led.color *= 0.98);

    let mut i = 0;
    for star in stars {
        let d = Vec2::new(star.x - center.x, star.y - center.y);
        let c = *buffers.get_buffer_item::<Rgb>("colors", i % 10)?;
        sled.modulate_at_dir(d, |led| {
            let d_sq = d.length_squared();
            led.color + (c / d_sq)
        });
        i += 1;
    }

    Ok(())
}

fn random_spawn_location(rng: &mut ThreadRng, center: Vec2) -> Vec2 {
    let sign = match rng.gen_bool(0.5) {
        true => 1.0,
        false => -1.0,
    };
    center + Vec2::new(rng.gen_range(0.85..15.0) * sign, rng.gen_range(40.0..250.0))
}
