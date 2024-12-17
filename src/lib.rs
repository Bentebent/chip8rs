use std::{
    thread,
    time::{
        Duration,
        Instant,
    },
};

use anyhow::Error;
use macroquad::{
    audio::{
        load_sound,
        play_sound,
        PlaySoundParams,
    },
    input::{
        is_key_pressed,
        KeyCode,
    },
};

mod constants;
mod emulator;
mod mem;
mod process;

pub async fn run(path: String, pixel_size: i32, window_size: (i32, i32)) -> Result<(), Error> {
    let rom = crate::mem::Rom::load(path)?;
    let sound = load_sound(r"assets/beep.wav").await?;
    play_sound(
        &sound,
        PlaySoundParams {
            looped: false,
            volume: 0.0, // Muted
        },
    );

    thread::sleep(Duration::new(1, 0));
    let mut emulator = emulator::Emulator::start(rom, pixel_size, window_size, sound);

    let start = Instant::now();
    let mut t = start - Duration::new(1337, 0);
    let mut t_sound = start - Duration::new(1337, 0);

    loop {
        let now = Instant::now();
        if now.duration_since(t_sound).as_secs_f64() * 1000.0 >= constants::MS_60HZ {
            t_sound = now;
            emulator.beep();
            emulator.tick_delay();
        }
        if now.duration_since(t).as_secs_f64() * 1000.0 >= constants::MS_PER_INSTRUCTION {
            t = now;
            emulator.run().await?;
        }
        emulator.render().await;
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
    }

    Ok(())
}
