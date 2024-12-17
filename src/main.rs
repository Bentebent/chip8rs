use macroquad::{
    miniquad,
    window::Conf,
};

const SCREEN_WIDTH: i32 = 64;
const SCREEN_HEIGHT: i32 = 32;

const PIXEL_SIZE: i32 = 10;

fn window_conf() -> Conf {
    Conf {
        window_title: String::from("chip8.rs"),
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
    #[allow(unused_variables)]
    let path = r"roms/IBM Logo.ch8";
    //let path = r"roms/test_flags.ch8";
    //let path = r"roms/test_opcode.ch8";
    //let path = r"roms/addition_problems.ch8";
    //let path = r"roms/random_number.ch8";
    //let path = r"roms/beep.ch8";
    //let path = r"roms/astro_dodge.ch8";
    if let Err(error) = chip8rs::run(path.into(), PIXEL_SIZE, (SCREEN_WIDTH, SCREEN_HEIGHT), &mut None).await {
        println!("Chip8 emulator failed in an unexpected manner: {}", error)
    }
}
