use std::{
    collections::HashMap,
    fs::File,
    io::Read,
};

use color_eyre::eyre::{
    eyre,
    Result,
};
use macroquad::{
    camera::{
        set_default_camera,
        Camera2D,
    },
    color::{
        self,
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

use crate::{
    constants,
    process,
};

#[allow(dead_code)]
pub enum Interpreter {
    CosmacVIP,
    Chip48,
    SuperChip,
}
pub struct ProgramCounter(usize);

impl ProgramCounter {
    pub fn inner(&self) -> &usize {
        &self.0
    }
    pub fn increment(&mut self) {
        self.0 += 2;
        self.0 %= constants::RAM_SIZE;
    }

    pub fn jump<T: Into<usize>>(&mut self, address: T) {
        self.0 = address.into();
    }
}

pub struct Ram {
    memory: [u8; constants::RAM_SIZE],
}

impl Ram {
    fn op_code(&self, pc: &ProgramCounter) -> u16 {
        (self.memory[*pc.inner()] as u16) << 8 | (self.memory[pc.inner() + 1] as u16)
    }

    pub fn reset_vram(&mut self) {
        self.memory[constants::DISPLAY_RANGE.0..constants::DISPLAY_RANGE.1].fill(0);
    }

    pub fn get<T: Into<usize>>(&self, index: T) -> u8 {
        *self.memory.get(index.into()).unwrap()
    }

    pub fn get_mut<T: Into<usize>>(&mut self, index: T) -> &mut u8 {
        self.memory.get_mut(index.into()).unwrap()
    }
}

pub struct Rom {
    data: Vec<u8>,
}

impl Rom {
    pub fn load(path: String) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut data = vec![];

        file.read_to_end(&mut data)?;

        if data.len() > constants::RAM_SIZE {
            return Err(eyre!(
                "ROM is larger than available memory, {} > {}",
                data.len(),
                constants::RAM_SIZE - constants::MEMORY_OFFSET
            ));
        }

        Ok(Self { data })
    }
}

impl From<Rom> for Ram {
    fn from(value: Rom) -> Self {
        let mut buffer = [0; constants::RAM_SIZE];
        let length = std::cmp::min(constants::RAM_SIZE, value.data.len());
        buffer[constants::MEMORY_OFFSET..constants::MEMORY_OFFSET + length].copy_from_slice(&value.data);

        Ram { memory: buffer }
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

impl InstructionData {
    #[allow(dead_code)]
    fn debug_print(&self) {
        println!("op_code: {:x}", self.op_code);
        println!("instruction: {:x}", self.instruction);

        println!("x: {}", self.x);
        println!("y: {}", self.y);
        println!("n: {}", self.n);
        println!("nn: {}", self.nn);
        println!("nnn: {:x}", self.nnn);
    }
}

pub struct Register {
    registers: HashMap<String, u8>,
}

impl Register {
    fn new() -> Self {
        let registers = HashMap::from([
            ("V0".into(), 0),
            ("V1".into(), 0),
            ("V2".into(), 0),
            ("V3".into(), 0),
            ("V4".into(), 0),
            ("V5".into(), 0),
            ("V6".into(), 0),
            ("V7".into(), 0),
            ("V8".into(), 0),
            ("V9".into(), 0),
            ("VA".into(), 0),
            ("VB".into(), 0),
            ("VC".into(), 0),
            ("VD".into(), 0),
            ("VE".into(), 0),
            ("VF".into(), 0),
        ]);

        Self { registers }
    }

    pub fn get(&self, key: &str) -> u8 {
        *self.registers.get(key).unwrap()
    }

    pub fn get_mut(&mut self, key: &str) -> &mut u8 {
        self.registers.get_mut(key).unwrap()
    }
}

pub struct Emulator {
    interpreter: Interpreter,
    memory: Ram,
    pc: ProgramCounter,
    stack: Vec<u16>,
    register: Register,
    index_register: u16,
    delay_timer: u8,
    sound_timer: u8,
    pixel_size: i32,
    window_size: (i32, i32),
    render_target: RenderTarget,
    camera: Camera2D,
}

impl Emulator {
    pub fn start(rom: Rom, pixel_size: i32, window_size: (i32, i32)) -> Self {
        let render_target = render_target((pixel_size * window_size.0) as u32, (pixel_size * window_size.1) as u32);
        render_target
            .texture
            .set_filter(macroquad::texture::FilterMode::Nearest);
        let mut camera = Camera2D::from_display_rect(Rect::new(0., 0., screen_width(), screen_height()));
        camera.render_target = Some(render_target.clone());

        Self {
            interpreter: Interpreter::SuperChip,
            memory: rom.into(),
            pc: ProgramCounter(constants::MEMORY_OFFSET),
            stack: vec![],
            register: Register::new(),
            index_register: 0,
            delay_timer: 0,
            sound_timer: 0,
            pixel_size,
            window_size,
            render_target,
            camera,
        }
    }

    pub async fn run(&mut self) {
        let op_code = self.memory.op_code(&self.pc);
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
        self.execute(instruction_data).await;
    }

    async fn execute(&mut self, instruction_data: InstructionData) {
        match (instruction_data.op_code, instruction_data.instruction) {
            (0x0000, _) => {}
            (0x00E0, _) => {
                process::op_00E0(&self.camera, color::BLACK, &mut self.memory);
            }
            (0x00EE, _) => {
                process::op_00EE(&mut self.pc, &mut self.stack);
            }
            (_, 0x1000) => {
                process::op_1NNN(&mut self.pc, instruction_data.nnn);
            }
            (_, 0x2000) => {
                process::op_2NNN(&mut self.stack, &mut self.pc, instruction_data.nnn);
            }
            (_, 0x3000) => {
                process::op_3XNN(&self.register, instruction_data.x, instruction_data.nn, &mut self.pc);
            }
            (_, 0x4000) => {
                process::op_4XNN(&self.register, instruction_data.x, instruction_data.nn, &mut self.pc);
            }
            (_, 0x5000) => {
                process::op_5XNN(&self.register, instruction_data.x, instruction_data.y, &mut self.pc);
            }
            (_, 0x6000) => {
                process::op_6XNN(&mut self.register, instruction_data.x, instruction_data.nn);
            }
            (_, 0x7000) => {
                process::op_7XNN(&mut self.register, instruction_data.x, instruction_data.nn);
            }
            (_, 0x8000) if instruction_data.n == 0x0 => {
                process::op_8XY0(&mut self.register, instruction_data.x, instruction_data.y);
            }
            (_, 0x8000) if instruction_data.n == 0x1 => {
                process::op_8XY1(&mut self.register, instruction_data.x, instruction_data.y);
            }
            (_, 0x8000) if instruction_data.n == 0x2 => {
                process::op_8XY2(&mut self.register, instruction_data.x, instruction_data.y);
            }
            (_, 0x8000) if instruction_data.n == 0x3 => {
                process::op_8XY3(&mut self.register, instruction_data.x, instruction_data.y);
            }
            (_, 0x8000) if instruction_data.n == 0x4 => {
                process::op_8XY4(&mut self.register, instruction_data.x, instruction_data.y);
            }
            (_, 0x8000) if instruction_data.n == 0x5 => {
                process::op_8XY5(&mut self.register, instruction_data.x, instruction_data.y);
            }
            (_, 0x8000) if instruction_data.n == 0x6 => {
                process::op_8XY6(
                    &self.interpreter,
                    &mut self.register,
                    instruction_data.x,
                    instruction_data.y,
                );
            }
            (_, 0x8000) if instruction_data.n == 0x7 => {
                process::op_8XY7(&mut self.register, instruction_data.x, instruction_data.y);
            }
            (_, 0x8000) if instruction_data.n == 0xE => {
                process::op_8XYE(
                    &self.interpreter,
                    &mut self.register,
                    instruction_data.x,
                    instruction_data.y,
                );
            }
            (_, 0x9000) => {
                process::op_9XY0(&self.register, instruction_data.x, instruction_data.y, &mut self.pc);
            }
            (_, 0xA000) => {
                process::op_ANNN(&mut self.index_register, instruction_data.nnn);
            }
            (_, 0xB000) => {
                process::op_BNNN(
                    &self.interpreter,
                    &self.register,
                    &mut self.pc,
                    instruction_data.x,
                    instruction_data.nnn,
                );
            }
            (_, 0xC000) => {
                process::op_CXNN(&mut self.register, instruction_data.x, instruction_data.nn);
            }
            (_, 0xD000) => {
                process::DXYN(
                    &mut self.memory,
                    &mut self.register,
                    self.index_register,
                    &self.camera,
                    &self.window_size,
                    self.pixel_size,
                    instruction_data,
                );
            }
            (_, 0xF000) if instruction_data.n == 0x7 => {
                process::op_FX07(&mut self.register, instruction_data.x, &self.delay_timer);
            }

            (_, 0xF000) if instruction_data.nn == 0xF => {
                process::op_FX15(&mut self.register, instruction_data.x, &mut self.delay_timer);
            }
            (_, 0xF000) if instruction_data.n == 0x12 => {
                process::op_FX18(&mut self.register, instruction_data.x, &mut self.sound_timer);
            }
            (_, 0xF000) if instruction_data.nn == 0x1E => {
                process::op_FX1E(&self.register, instruction_data.x, &mut self.index_register);
            }
            (_, 0xF000) if instruction_data.op_code & 0xF0FF == 0xF033 => {
                process::op_FX33(
                    &self.register,
                    &mut self.memory,
                    instruction_data.x,
                    self.index_register,
                );
            }
            (_, 0xF000) if instruction_data.op_code & 0xF0FF == 0xF055 => {
                process::op_FX55(
                    &self.interpreter,
                    &self.register,
                    &mut self.memory,
                    &mut self.index_register,
                    instruction_data.x,
                );
            }
            (_, 0xF000) if instruction_data.op_code & 0xF0FF == 0xF065 => {
                process::op_FX65(
                    &self.interpreter,
                    &mut self.register,
                    &self.memory,
                    &mut self.index_register,
                    instruction_data.x,
                );
            }
            _ => println!("Instruction not implemented: {:x}", instruction_data.op_code),
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
}
