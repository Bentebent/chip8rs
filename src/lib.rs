use std::time::{
    SystemTime,
    UNIX_EPOCH,
};

use color_eyre::eyre::Result;
use macroquad::{
    audio::load_sound,
    input::{
        is_key_pressed,
        KeyCode,
    },
};

mod constants;
mod emulator;
mod process;

pub async fn run(path: String, pixel_size: i32, window_size: (i32, i32)) -> Result<()> {
    let rom = emulator::Rom::load(path)?;
    let sound = load_sound(r"assets/beep.wav").await?;
    let mut emulator = emulator::Emulator::start(rom, pixel_size, window_size, sound);
    let mut t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as f64;
    let mut t_sound = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as f64;

    loop {
        let t2 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as f64;
        if t2 - t_sound > constants::MS_60HZ {
            t_sound = t2;
            emulator.beep();
        }
        if t2 - t > constants::MS_PER_INSTRUCTION {
            emulator.run().await;
            t = t2;

            emulator.render().await
        }

        if is_key_pressed(KeyCode::Escape) {
            break;
        }
    }

    Ok(())
}
