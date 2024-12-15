use macroquad::{
    miniquad,
    window::Conf,
};

const SCREEN_WIDTH: i32 = 64;
const SCREEN_HEIGHT: i32 = 32;

const PIXEL_SIZE: i32 = 10;

fn window_conf() -> Conf {
    Conf {
        window_title: "chip8.rs".to_owned(),
        fullscreen: false,
        window_resizable: false,
        window_width: SCREEN_WIDTH * PIXEL_SIZE,
        window_height: SCREEN_HEIGHT * PIXEL_SIZE,
        platform: miniquad::conf::Platform { ..Default::default() },
        ..Default::default()
    }
}
#[macroquad::main(window_conf)]
async fn main() {
    color_eyre::install().expect("Failed to initialize color_eyre error handler");

    #[allow(unused_variables)]
    let path = r"roms/IBM Logo.ch8";
    let path = r"roms/test_opcode.ch8";
    if let Err(error) = chip8rs::run(path.into(), PIXEL_SIZE, (SCREEN_WIDTH, SCREEN_HEIGHT)).await {
        println!("Chip8 emulator failed in an unexpected manner: {}", error)
    }
}
