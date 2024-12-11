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
const RAM_SIZE: usize = 4096;
const INSTRUCTIONS_PER_SECOND: usize = 1;
const MS_PER_INSTRUCTION: u128 = (1000 / INSTRUCTIONS_PER_SECOND) as u128;

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
                RAM_SIZE
            ));
        }

        Ok(Self { data })
    }

    pub fn load_into_memory(self) -> [u8; 4096] {
        let mut buffer = [0; RAM_SIZE];
        let length = std::cmp::min(RAM_SIZE, self.data.len());
        buffer[..length].copy_from_slice(&self.data[..length]);

        buffer
    }
}
#[derive(Debug)]
struct InstructionData {
    pub op_code: u16,
    pub instruction: u16,
    pub x: String,
    pub y: u16,
    pub n: u16,
    pub nn: u16,
    pub nnn: u16,
}

impl InstructionData {
    fn print(&self) {
        println!("op_code: {:x}", self.op_code);
        println!("instruction: {:x}", self.instruction);

        println!("x: {}", self.x);
        println!("y: {:x}", self.y);
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
}

impl Emulator {
    fn start(rom: ROM) -> Self {
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
        Self {
            memory: rom.load_into_memory(),
            pc: ProgramCounter(0),
            stack: vec![],
            registers,
            index_register: 0,
        }
    }

    fn run(&mut self) {
        let op_code = (self.memory[*self.pc.inner()] as u16) << 8 | (self.memory[self.pc.inner() + 1] as u16);
        self.pc.increment();

        let instruction_data = InstructionData {
            op_code,
            instruction: op_code & 0xF000,
            x: format!("V{}", (op_code & 0x0F00) >> 8),
            y: op_code & 0x00F0,
            n: op_code & 0x000F,
            nn: op_code & 0x00FF,
            nnn: op_code & 0x0FFF,
        };
        self.execute(instruction_data);
    }

    fn execute(&mut self, instruction_data: InstructionData) {
        match (instruction_data.op_code, instruction_data.instruction) {
            (0x0000, _) => {}
            (0x00E0, _) => {
                println!("Clear screen");
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
                println!("Draw sprite");
            }
            _ => println!("Instruction not implemented: {:x}", instruction_data.instruction),
        }
    }
}

pub fn run(path: String) -> Result<()> {
    let rom = ROM::load(path)?;
    let mut emulator = Emulator::start(rom);
    let mut t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    loop {
        let t2 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        if t2 - t > MS_PER_INSTRUCTION {
            emulator.run();
            t = t2;
        }
    }

    Ok(())
}
