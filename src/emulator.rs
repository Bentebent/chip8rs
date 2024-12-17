use std::collections::HashMap;

use macroquad::{
    audio::{
        stop_sound,
        Sound,
    },
    camera::{
        set_default_camera,
        Camera2D,
    },
    color,
    input::{
        is_key_down,
        is_key_released,
        KeyCode,
    },
    math::vec2,
    prelude::{
        next_frame,
        render_target,
        Rect,
    },
    texture::{
        draw_texture_ex,
        DrawTextureParams,
        RenderTarget,
    },
    window::{
        screen_height,
        screen_width,
    },
};
use thiserror::Error;

use crate::{
    constants,
    mem::{
        AddressStack,
        Ram,
        RamError,
        Register,
        Rom,
    },
    process::{
        self,
        ProcessingError,
    },
};

#[rustfmt::skip]
const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9

    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

#[allow(dead_code)]
pub enum Interpreter {
    CosmacVIP,
    Chip48,
    SuperChip,
}

#[derive(Debug, Clone)]
pub struct ProgramCounter(usize);

impl ProgramCounter {
    pub fn inner(&self) -> &usize {
        &self.0
    }
    pub fn increment(&mut self) {
        self.0 += 2;
    }

    pub fn decrement(&mut self) {
        self.0 -= 2;
    }

    pub fn jump<T: Into<usize>>(&mut self, address: T) {
        self.0 = address.into();
    }
}

#[derive(Debug)]
pub struct InstructionData {
    pub op_code: u16,
    pub instruction: u16,
    pub x: String,
    pub y: String,
    pub n: u16,
    pub nn: u8,
    pub nnn: u16,
}

pub struct KeyPad {
    key_code_hex_mapping: HashMap<u8, KeyCode>,
}

impl KeyPad {
    fn new() -> Self {
        let key_code_hex_mapping: HashMap<u8, KeyCode> = HashMap::from([
            (0x1, KeyCode::Key1),
            (0x2, KeyCode::Key2),
            (0x3, KeyCode::Key3),
            (0xC, KeyCode::Key4),
            (0x4, KeyCode::Q),
            (0x5, KeyCode::W),
            (0x6, KeyCode::E),
            (0xD, KeyCode::R),
            (0x7, KeyCode::A),
            (0x8, KeyCode::S),
            (0x9, KeyCode::D),
            (0xE, KeyCode::F),
            (0xA, KeyCode::Z),
            (0x0, KeyCode::X),
            (0xB, KeyCode::C),
            (0xF, KeyCode::V),
        ]);

        Self { key_code_hex_mapping }
    }

    pub fn get_key_released(&self) -> Option<u8> {
        self.key_code_hex_mapping
            .iter()
            .find(|(_, code)| is_key_released(**code))
            .map(|(hex, _)| *hex)
    }
    pub fn is_key_pressed(&self, hex: u8) -> bool {
        if let Some(key_code) = self.key_code_hex_mapping.get(&hex) {
            is_key_down(*key_code)
        } else {
            false
        }
    }
}

#[derive(Error, Debug)]
pub enum EmulatorError {
    #[error("failed processing op code 0x{:04X}", op_code)]
    OpError { source: ProcessingError, op_code: u16 },

    #[error("failed to fetch instruction 0x{:04X}", pc.inner())]
    PCInvalid { pc: ProgramCounter, source: RamError },

    #[error("failed renderingop code 0x{:04X}", op_code)]
    RenderingFailed { source: ProcessingError, op_code: u16 },
}

impl EmulatorError {
    fn from_processing_error(source: ProcessingError, op_code: u16) -> EmulatorError {
        match op_code {
            val if (val & 0xF000) == 0xD000 => EmulatorError::RenderingFailed { source, op_code },
            _ => EmulatorError::OpError { source, op_code },
        }
    }
}

pub struct Emulator {
    interpreter: Interpreter,
    memory: Ram,
    pc: ProgramCounter,
    stack: AddressStack,
    register: Register,
    index_register: u16,
    delay_timer: u8,
    sound_timer: u8,
    keypad: KeyPad,
    pixel_size: i32,
    window_size: (i32, i32),
    render_target: RenderTarget,
    camera: Camera2D,
    sound: Sound,
}

impl Emulator {
    pub fn start(rom: Rom, pixel_size: i32, window_size: (i32, i32), beep: Sound) -> Self {
        let render_target = render_target((pixel_size * window_size.0) as u32, (pixel_size * window_size.1) as u32);
        render_target
            .texture
            .set_filter(macroquad::texture::FilterMode::Nearest);
        let mut camera = Camera2D::from_display_rect(Rect::new(0., 0., screen_width(), screen_height()));
        camera.render_target = Some(render_target.clone());

        Self {
            interpreter: Interpreter::SuperChip,
            memory: Ram::load(rom, &FONT),
            pc: ProgramCounter(constants::MEMORY_OFFSET),
            stack: AddressStack::default(),
            register: Register::new(),
            index_register: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: KeyPad::new(),
            pixel_size,
            window_size,
            render_target,
            camera,
            sound: beep,
        }
    }

    pub async fn run(&mut self) -> Result<(), EmulatorError> {
        let op_code = self.memory.op_code(&self.pc).map_err(|err| EmulatorError::PCInvalid {
            pc: self.pc.clone(),
            source: err,
        })?;

        self.pc.increment();

        let instruction_data = InstructionData {
            op_code,
            instruction: op_code & 0xF000,
            x: format!("V{:X}", (op_code & 0x0F00) >> 8),
            y: format!("V{:X}", (op_code & 0x00F0) >> 4),
            n: op_code & 0x000F,
            nn: (op_code & 0x00FF) as u8,
            nnn: op_code & 0x0FFF,
        };
        self.execute(instruction_data)
            .map_err(|err| EmulatorError::from_processing_error(err, op_code))
    }

    fn execute(&mut self, instruction_data: InstructionData) -> Result<(), ProcessingError> {
        match (instruction_data.op_code, instruction_data.instruction) {
            (0x0000, _) => {}
            (0x00E0, _) => process::op_00E0(&self.camera, color::BLACK, &mut self.memory),
            (0x00EE, _) => process::op_00EE(&mut self.pc, &mut self.stack)?,
            (_, 0x1000) => process::op_1NNN(&mut self.pc, instruction_data.nnn),
            (_, 0x2000) => process::op_2NNN(&mut self.stack, &mut self.pc, instruction_data.nnn),
            (_, 0x3000) => process::op_3XNN(&self.register, instruction_data.x, instruction_data.nn, &mut self.pc)?,
            (_, 0x4000) => process::op_4XNN(&self.register, instruction_data.x, instruction_data.nn, &mut self.pc)?,
            (_, 0x5000) => process::op_5XNN(&self.register, instruction_data.x, instruction_data.y, &mut self.pc)?,
            (_, 0x6000) => process::op_6XNN(&mut self.register, instruction_data.x, instruction_data.nn)?,
            (_, 0x7000) => process::op_7XNN(&mut self.register, instruction_data.x, instruction_data.nn)?,
            (_, 0x8000) if instruction_data.n == 0x0 => {
                process::op_8XY0(&mut self.register, instruction_data.x, instruction_data.y)?
            }
            (_, 0x8000) if instruction_data.n == 0x1 => {
                process::op_8XY1(&mut self.register, instruction_data.x, instruction_data.y)?
            }
            (_, 0x8000) if instruction_data.n == 0x2 => {
                process::op_8XY2(&mut self.register, instruction_data.x, instruction_data.y)?
            }

            (_, 0x8000) if instruction_data.n == 0x3 => {
                process::op_8XY3(&mut self.register, instruction_data.x, instruction_data.y)?
            }
            (_, 0x8000) if instruction_data.n == 0x4 => {
                process::op_8XY4(&mut self.register, instruction_data.x, instruction_data.y)?
            }
            (_, 0x8000) if instruction_data.n == 0x5 => {
                process::op_8XY5(&mut self.register, instruction_data.x, instruction_data.y)?
            }
            (_, 0x8000) if instruction_data.n == 0x6 => process::op_8XY6(
                &self.interpreter,
                &mut self.register,
                instruction_data.x,
                instruction_data.y,
            )?,

            (_, 0x8000) if instruction_data.n == 0x7 => {
                process::op_8XY7(&mut self.register, instruction_data.x, instruction_data.y)?
            }
            (_, 0x8000) if instruction_data.n == 0xE => process::op_8XYE(
                &self.interpreter,
                &mut self.register,
                instruction_data.x,
                instruction_data.y,
            )?,
            (_, 0x9000) => process::op_9XY0(&self.register, instruction_data.x, instruction_data.y, &mut self.pc)?,
            (_, 0xA000) => {
                process::op_ANNN(&mut self.index_register, instruction_data.nnn);
            }
            (_, 0xB000) => process::op_BNNN(
                &self.interpreter,
                &self.register,
                &mut self.pc,
                instruction_data.x,
                instruction_data.nnn,
            )?,
            (_, 0xC000) => process::op_CXNN(&mut self.register, instruction_data.x, instruction_data.nn)?,
            (_, 0xD000) => process::DXYN(
                &mut self.memory,
                &mut self.register,
                self.index_register,
                &self.camera,
                &self.window_size,
                self.pixel_size,
                instruction_data,
            )?,
            (_, 0xE000) if instruction_data.op_code & 0xF0FF == 0xE09E => {
                process::op_EX9E(&self.register, &self.keypad, &mut self.pc, instruction_data.x)?
            }
            (_, 0xE000) if instruction_data.op_code & 0xF0FF == 0xE0A1 => {
                process::op_EXA1(&self.register, &self.keypad, &mut self.pc, instruction_data.x)?
            }
            (_, 0xF000) if instruction_data.op_code & 0xF0FF == 0xF007 => {
                process::op_FX07(&mut self.register, instruction_data.x, &self.delay_timer)?
            }

            (_, 0xF000) if instruction_data.op_code & 0xF0FF == 0xF015 => {
                process::op_FX15(&mut self.register, instruction_data.x, &mut self.delay_timer)?
            }
            (_, 0xF000) if instruction_data.nn == 0x1E => {
                process::op_FX1E(&self.register, instruction_data.x, &mut self.index_register)?
            }
            (_, 0xF000) if instruction_data.op_code & 0xF0FF == 0xF00A => {
                process::op_FX0A(&mut self.register, &mut self.pc, &self.keypad, instruction_data.x)?
            }
            (_, 0xF000) if instruction_data.op_code & 0xF0FF == 0xF018 => process::op_FX18(
                &mut self.register,
                instruction_data.x,
                &mut self.sound_timer,
                &self.sound,
            )?,
            (_, 0xF000) if instruction_data.op_code & 0xF0FF == 0xF029 => {
                process::op_FX29(&self.register, &mut self.index_register, instruction_data.x)?
            }
            (_, 0xF000) if instruction_data.op_code & 0xF0FF == 0xF033 => process::op_FX33(
                &self.register,
                &mut self.memory,
                instruction_data.x,
                self.index_register,
            )?,
            (_, 0xF000) if instruction_data.op_code & 0xF0FF == 0xF055 => process::op_FX55(
                &self.interpreter,
                &self.register,
                &mut self.memory,
                &mut self.index_register,
                instruction_data.x,
            )?,
            (_, 0xF000) if instruction_data.op_code & 0xF0FF == 0xF065 => process::op_FX65(
                &self.interpreter,
                &mut self.register,
                &self.memory,
                &mut self.index_register,
                instruction_data.x,
            )?,
            _ => println!("Instruction not implemented: {:x}", instruction_data.op_code),
        }
        Ok(())
    }

    pub fn beep(&mut self) {
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        } else {
            stop_sound(&self.sound);
        }
    }
    pub fn tick_delay(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
    }
    pub async fn render(&self) {
        set_default_camera();
        draw_texture_ex(
            &self.render_target.texture,
            0.,
            0.,
            macroquad::color::WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                flip_y: true,
                ..Default::default()
            },
        );

        next_frame().await
    }

    pub fn export_render_target(&self, path: &str) {
        self.render_target.texture.get_texture_data().export_png(path);
    }
}
