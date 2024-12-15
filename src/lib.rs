use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    time::{
        SystemTime,
        UNIX_EPOCH,
    },
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
        GOLD,
    },
    math::vec2,
    miniquad::start,
    prelude::{
        clear_background,
        draw_rectangle,
        next_frame,
        render_target,
        scene::clear,
        set_camera,
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
const RAM_SIZE: usize = 4096;
const INSTRUCTIONS_PER_SECOND: usize = 1;
const MS_PER_INSTRUCTION: u128 = (1000 / INSTRUCTIONS_PER_SECOND) as u128;
const MEMORY_OFFSET: usize = 512;

struct ProgramCounter(usize);

impl ProgramCounter {
    fn inner(&self) -> &usize {
        &self.0
    }
    fn increment(&mut self) {
        self.0 += 2;
        self.0 %= RAM_SIZE;
    }

    fn jump(&mut self, address: usize) {
        self.0 = address;
    }
}

pub struct ROM {
    data: Vec<u8>,
}

impl ROM {
    pub fn load(path: String) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut data = vec![];

        file.read_to_end(&mut data)?;

        if data.len() > RAM_SIZE {
            return Err(eyre!(
                "ROM is larger than available memory, {} > {}",
                data.len(),
                RAM_SIZE - MEMORY_OFFSET
            ));
        }

        Ok(Self { data })
    }

    pub fn load_into_memory(self) -> [u8; 4096] {
        let mut buffer = [0; RAM_SIZE];
        let length = std::cmp::min(RAM_SIZE, self.data.len());
        buffer[MEMORY_OFFSET..MEMORY_OFFSET + length].copy_from_slice(&self.data);

        buffer
    }
}
#[derive(Debug)]
struct InstructionData {
    pub op_code: u16,
    pub instruction: u16,
    pub x: String,
    pub y: String,
    pub n: u16,
    pub nn: u16,
    pub nnn: u16,
}

impl InstructionData {
    fn print(&self) {
        println!("op_code: {:x}", self.op_code);
        println!("instruction: {:x}", self.instruction);

        println!("x: {}", self.x);
        println!("y: {}", self.y);
        println!("n: {}", self.n);
        println!("nn: {}", self.nn);
        println!("nnn: {:x}", self.nnn);
    }
}

struct Emulator {
    memory: [u8; RAM_SIZE],
    pc: ProgramCounter,
    stack: Vec<u16>,
    registers: HashMap<String, u16>,
    index_register: u16,
    pixel_size: i32,
    window_size: (i32, i32),
    render_target: RenderTarget,
    camera: Camera2D,
}

impl Emulator {
    fn start(rom: ROM, pixel_size: i32, window_size: (i32, i32)) -> Self {
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

        let render_target = render_target((pixel_size * window_size.0) as u32, (pixel_size * window_size.1) as u32);
        render_target
            .texture
            .set_filter(macroquad::texture::FilterMode::Nearest);
        let mut camera = Camera2D::from_display_rect(Rect::new(0., 0., screen_width(), screen_height()));
        camera.render_target = Some(render_target.clone());

        Self {
            memory: rom.load_into_memory(),
            pc: ProgramCounter(MEMORY_OFFSET),
            stack: vec![],
            registers,
            index_register: 0,
            pixel_size,
            window_size,
            render_target,
            camera,
        }
    }

    async fn run(&mut self) {
        let op_code = (self.memory[*self.pc.inner()] as u16) << 8 | (self.memory[self.pc.inner() + 1] as u16);
        self.pc.increment();

        let instruction_data = InstructionData {
            op_code,
            instruction: op_code & 0xF000,
            x: format!("V{:x}", (op_code & 0x0F00) >> 8),
            y: format!("V{:x}", (op_code & 0x00F0) >> 4),
            n: op_code & 0x000F,
            nn: op_code & 0x00FF,
            nnn: op_code & 0x0FFF,
        };
        self.execute(instruction_data).await;
    }

    async fn execute(&mut self, instruction_data: InstructionData) {
        match (instruction_data.op_code, instruction_data.instruction) {
            (0x0000, _) => {}
            (0x00E0, _) => {
                println!("Clear screen");
                set_camera(&self.camera);
                clear_background(color::BLACK);
            }
            (_, 0x1000) => {
                println!("Set PC to {}", instruction_data.nnn);
                self.pc.jump(instruction_data.nnn as usize);
            }
            (_, 0x6000) => {
                println!("Set register {} to {}", instruction_data.x, instruction_data.nn);
                *self.registers.get_mut(&instruction_data.x).unwrap() = instruction_data.nn;
            }
            (_, 0x7000) => {
                println!("Add value {} to register {}", instruction_data.nn, instruction_data.x);
                *self.registers.get_mut(&instruction_data.x).unwrap() += instruction_data.nn;
            }
            (_, 0xA000) => {
                println!("Set index register to {}", instruction_data.nnn);
                self.index_register = instruction_data.nnn;
            }
            (_, 0xD000) => {
                let start_x: i32 =
                    (*self.registers.get(&instruction_data.x).unwrap() as i32 % self.window_size.0) * self.pixel_size;

                let start_y: i32 =
                    (*self.registers.get(&instruction_data.y).unwrap() as i32 % self.window_size.1) * self.pixel_size;
                println!(
                    "Draw sprite {} at position ({:?}, {:?} with size {})",
                    self.index_register, start_x, start_y, instruction_data.n
                );

                set_camera(&self.camera);
                let size = instruction_data.n as usize;
                for y in 0..size {
                    let index = self.index_register as usize + y;
                    let sprite = *self.memory.get(index).unwrap();
                    for x in (0..u8::BITS).rev() {
                        let bit = (sprite >> x) & 1;
                        //                        let bit = (sprite >> x) & 1;
                        if bit == 1 {
                            draw_rectangle(
                                (start_x + (u8::BITS - x) as i32 * self.pixel_size) as f32,
                                (start_y + (y as i32) * self.pixel_size) as f32,
                                self.pixel_size as f32,
                                self.pixel_size as f32,
                                color::Color {
                                    r: 0.,
                                    g: 128.0,
                                    b: 0.,
                                    a: 1.,
                                },
                            );
                        }
                    }
                }
            }
            _ => println!("Instruction not implemented: {:x}", instruction_data.instruction),
        }
    }
}

pub async fn run(path: String, pixel_size: i32, window_size: (i32, i32)) -> Result<()> {
    let rom = ROM::load(path)?;
    let mut emulator = Emulator::start(rom, pixel_size, window_size);
    let mut t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    loop {
        let t2 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        if t2 - t > MS_PER_INSTRUCTION {
            emulator.run().await;
            t = t2;

            set_default_camera();
            draw_texture_ex(
                &emulator.render_target.texture,
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

    Ok(())
}
