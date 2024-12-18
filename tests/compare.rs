mod compare {

    use std::{
        fs,
        path::Path,
    };

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
        let path = Path::new(path);
        if let Some(folders) = path.parent() {
            let _ = fs::create_dir_all(folders);
        }
        emulator.export_render_target(path.to_str().unwrap());
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
    async fn comparison_tests() {
        let generated_identifier: String = env::var("GIT_SHA").unwrap_or("local".to_string());
        let tolerance = 0.0001;
        compare_chip8_logo(generated_identifier.clone(), tolerance).await;
        compare_ibm(generated_identifier.clone(), tolerance).await;
        compare_corax(generated_identifier.clone(), tolerance).await;
        compare_flags(generated_identifier.clone(), tolerance).await;
    }

    async fn compare_chip8_logo(generated_identifier: String, tolerance: f64) {
        let path = r"assets/roms/test/1-chip8-logo.ch8";
        let mut events = Some(vec![RunnerEvent::new(chip8rs::Trigger::TimerSeconds(2.0), {
            let generated_identifier = generated_identifier.clone();
            Box::new(move |emulator| {
                save_screenshot(
                    emulator,
                    &format!("tests/generated/1-chip8-logo/{}.png", generated_identifier),
                )
            })
        })]);

        run_emulator(path, &mut events).await;

        let baseline = image::open("tests/baseline/1-chip8-logo.png").unwrap();
        let generated = image::open(format!("tests/generated/1-chip8-logo/{}.png", generated_identifier)).unwrap();

        let comparison_result = compare::compare_images(baseline, generated);

        assert!(
            1.0 - &comparison_result.score < tolerance,
            "Chip8 logo comparison score too low: {}",
            comparison_result.score
        );
    }

    async fn compare_ibm(generated_identifier: String, tolerance: f64) {
        let path = r"assets/roms/test/IBM Logo.ch8";
        let mut events = Some(vec![RunnerEvent::new(chip8rs::Trigger::TimerSeconds(2.0), {
            let generated_identifier = generated_identifier.clone();
            Box::new(move |emulator| {
                save_screenshot(
                    emulator,
                    &format!("tests/generated/IBM Logo/{}.png", generated_identifier),
                )
            })
        })]);

        run_emulator(path, &mut events).await;

        let baseline = image::open("tests/baseline/IBM Logo.png").unwrap();
        let generated = image::open(format!("tests/generated/IBM Logo/{}.png", generated_identifier)).unwrap();

        let comparison_result = compare::compare_images(baseline, generated);

        assert!(
            1.0 - &comparison_result.score < tolerance,
            "IBM logo comparison score too low: {}",
            comparison_result.score
        );
    }

    async fn compare_corax(generated_identifier: String, tolerance: f64) {
        let path = r"assets/roms/test/3-corax+.ch8";
        let mut events = Some(vec![RunnerEvent::new(chip8rs::Trigger::TimerSeconds(2.0), {
            let generated_identifier = generated_identifier.clone();
            Box::new(move |emulator| {
                save_screenshot(
                    emulator,
                    &format!("tests/generated/3-corax+/{}.png", generated_identifier),
                )
            })
        })]);

        run_emulator(path, &mut events).await;

        let baseline = image::open("tests/baseline/corax.png").unwrap();
        let generated = image::open(format!("tests/generated/3-corax+/{}.png", generated_identifier)).unwrap();

        let comparison_result = compare::compare_images(baseline, generated);

        assert!(
            1.0 - &comparison_result.score < tolerance,
            "Corax+ comparison score too low: {}",
            comparison_result.score
        );
    }

    async fn compare_flags(generated_identifier: String, tolerance: f64) {
        let path = r"assets/roms/test/4-flags.ch8";
        let mut events = Some(vec![RunnerEvent::new(chip8rs::Trigger::TimerSeconds(2.0), {
            let generated_identifier = generated_identifier.clone();
            Box::new(move |emulator| {
                save_screenshot(
                    emulator,
                    &format!("tests/generated/4-flags/{}.png", generated_identifier),
                )
            })
        })]);

        run_emulator(path, &mut events).await;

        let baseline = image::open("tests/baseline/4-flags.png").unwrap();
        let generated = image::open(format!("tests/generated/4-flags/{}.png", generated_identifier)).unwrap();

        let comparison_result = compare::compare_images(baseline, generated);

        assert!(
            1.0 - &comparison_result.score < tolerance,
            "Flags comparison score too low: {}",
            comparison_result.score
        );
    }
}
