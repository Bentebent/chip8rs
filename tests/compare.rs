mod compare {

    use chip8rs::{
        emulator,
        RunnerEvent,
    };
    use image::DynamicImage;
    use image_compare::{
        Algorithm,
        Similarity,
    };
    use macroquad::{
        miniquad::window::set_fullscreen,
        window::{
            next_frame,
            request_new_screen_size,
        },
    };

    pub const SCREEN_WIDTH: i32 = 64;
    pub const SCREEN_HEIGHT: i32 = 32;
    pub const PIXEL_SIZE: i32 = 10;

    pub async fn set_window_conf() {
        set_fullscreen(false);
        request_new_screen_size((SCREEN_WIDTH * PIXEL_SIZE) as f32, (SCREEN_HEIGHT * PIXEL_SIZE) as f32);

        next_frame().await;
        next_frame().await;
    }

    pub async fn run_emulator(rom_path: &str, events: &mut Option<Vec<RunnerEvent>>) {
        set_window_conf().await;

        if (chip8rs::run(rom_path.into(), PIXEL_SIZE, (SCREEN_WIDTH, SCREEN_HEIGHT), events).await).is_err() {
            panic!();
        }
    }

    pub fn compare_images(first: DynamicImage, second: DynamicImage) -> Similarity {
        image_compare::rgb_similarity_structure(&Algorithm::MSSIMSimple, &first.into_rgb8(), &second.into_rgb8())
            .unwrap()
    }

    pub fn save_screenshot(emulator: &emulator::Emulator, path: &str) {
        emulator.export_render_target(path);
    }
}
#[cfg(test)]
mod test {

    use std::env;

    use chip8rs::RunnerEvent;

    use crate::compare::{
        self,
        run_emulator,
        save_screenshot,
    };

    #[macroquad::test]
    async fn compare_ibm() {
        env::set_var("RUST_BACKTRACE", "1");
        let generated_identifier: String = env::var("GIT_SHA").unwrap_or("local".to_string());
        let path = r"assets/roms/IBM Logo.ch8";
        let mut events = Some(vec![RunnerEvent::new(chip8rs::Trigger::TimerSeconds(2.0), {
            let generated_identifier = generated_identifier.clone();
            Box::new(move |emulator| {
                save_screenshot(emulator, &format!("tests/generated/ibm.{}.png", generated_identifier))
            })
        })]);

        run_emulator(path, &mut events).await;

        let baseline = image::open("tests/baseline/ibm.png").unwrap();
        let generated = image::open(format!("tests/generated/ibm.{}.png", generated_identifier)).unwrap();

        let comparison_result = compare::compare_images(baseline, generated);

        assert_eq!(comparison_result.score, 1.0);
    }
}
