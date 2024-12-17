use std::{
    path::Path,
    thread,
    time::{
        Duration,
        Instant,
        SystemTime,
        UNIX_EPOCH,
    },
};

use anyhow::Error;
use emulator::Emulator;
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
pub mod emulator;
mod mem;
mod process;

#[derive(Debug, Clone, Copy)]
pub enum Trigger {
    TimerSeconds(f64),
    InstructionCount(usize),
}

pub struct RunnerEvent {
    trigger: Trigger,
    on_trigger: Box<dyn Fn(&Emulator)>,
}

impl RunnerEvent {
    pub fn new(trigger: Trigger, on_trigger: Box<dyn Fn(&Emulator)>) -> Self {
        RunnerEvent { trigger, on_trigger }
    }
}

async fn scaffold(path: &str, pixel_size: i32, window_size: (i32, i32)) -> Result<emulator::Emulator, Error> {
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
    Ok(emulator::Emulator::start(rom, pixel_size, window_size, sound))
}

pub async fn run(
    path: String,
    pixel_size: i32,
    window_size: (i32, i32),
    events: &mut Option<Vec<RunnerEvent>>,
) -> Result<(), Error> {
    let mut emulator = scaffold(&path, pixel_size, window_size).await?;

    let mut start = Instant::now();
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

        if is_key_pressed(KeyCode::P) {
            let name = format!(
                ".dev/{}_{}.png",
                Path::new(&path).file_stem().unwrap().to_string_lossy(),
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
            );
            emulator.export_render_target(&name);
            println!("Printed screenshot at {}", name);
        }

        if let Some(events) = events {
            if let Some(current_event) = events.last() {
                match current_event.trigger {
                    Trigger::TimerSeconds(seconds) => {
                        if now.duration_since(start).as_secs_f64() > seconds {
                            (current_event.on_trigger)(&emulator);
                            events.pop();
                            start = now;
                        }
                    }
                    Trigger::InstructionCount(_) => todo!(),
                }
            } else {
                break;
            }
        }

        if is_key_pressed(KeyCode::Escape) {
            break;
        }
    }

    Ok(())
}
