use std::time::{
    SystemTime,
    UNIX_EPOCH,
};

use color_eyre::eyre::Result;
use macroquad::{
    self,
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
    let mut emulator = emulator::Emulator::start(rom, pixel_size, window_size);
    let mut t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    loop {
        let t2 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
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
