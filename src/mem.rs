use std::{
    collections::HashMap,
    fs::File,
    io::{
        self,
        Read,
    },
};

use thiserror::Error;

use crate::{
    constants,
    emulator::ProgramCounter,
};

#[derive(Error, Debug)]
pub(crate) enum RamError {
    #[error("invalid address {0}")]
    InvalidAddress(usize),
}

pub(crate) struct Ram {
    memory: [u8; constants::TOTAL_RAM],
}

impl Ram {
    pub fn load(rom: Rom, font: &[u8]) -> Self {
        let mut ram: Ram = rom.into();
        ram.memory[0..font.len()].copy_from_slice(font);

        ram
    }
    pub fn op_code(&self, pc: &ProgramCounter) -> Result<u16, RamError> {
        let pc = *pc.inner();
        let high = *self.memory.get(pc).ok_or(RamError::InvalidAddress(pc))? as u16;
        let low = *self.memory.get(pc + 1).ok_or(RamError::InvalidAddress(pc + 1))? as u16;
        Ok((high << 8) | low)
    }

    pub fn reset_vram(&mut self) {
        self.memory[constants::DISPLAY_RANGE.0..constants::DISPLAY_RANGE.1].fill(0);
    }

    pub fn get<T: Into<usize>>(&self, index: T) -> Result<u8, RamError> {
        let idx = index.into();
        self.memory.get(idx).ok_or(RamError::InvalidAddress(idx)).copied()
    }

    pub fn get_mut<T: Into<usize>>(&mut self, index: T) -> Result<&mut u8, RamError> {
        let idx = index.into();
        self.memory.get_mut(idx).ok_or(RamError::InvalidAddress(idx))
    }
}

impl From<Rom> for Ram {
    fn from(value: Rom) -> Self {
        let mut buffer = [0; constants::TOTAL_RAM];
        let length = std::cmp::min(constants::AVAILABLE_RAM, value.len());
        buffer[constants::MEMORY_OFFSET..constants::MEMORY_OFFSET + length].copy_from_slice(value.data());

        Ram { memory: buffer }
    }
}

#[derive(Error, Debug)]
pub(crate) enum RegisterError {
    #[error("address `{0}` is not a valid register")]
    InvalidAddress(String),
}

pub(crate) struct Register {
    registers: HashMap<String, u8>,
}

impl Register {
    pub fn new() -> Self {
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

    pub fn get(&self, key: &str) -> Result<u8, RegisterError> {
        self.registers
            .get(key)
            .copied()
            .ok_or_else(|| RegisterError::InvalidAddress(key.to_owned()))
    }

    pub fn set(&mut self, key: &str, val: u8) -> Result<(), RegisterError> {
        let register = self
            .registers
            .get_mut(key)
            .ok_or_else(|| RegisterError::InvalidAddress(key.to_owned()))?;
        *register = val;

        Ok(())
    }

    pub fn set_x_y(&mut self, x: &str, y: &str) -> Result<(), RegisterError> {
        let y_val = *self
            .registers
            .get(y)
            .ok_or_else(|| RegisterError::InvalidAddress(y.to_owned()))?;
        let x_val = self
            .registers
            .get_mut(x)
            .ok_or_else(|| RegisterError::InvalidAddress(x.to_owned()))?;

        *x_val = y_val;
        Ok(())
    }

    pub fn cmp_registers(&self, x: &str, y: &str) -> Result<bool, RegisterError> {
        Ok(self.get(x)? == self.get(y)?)
    }

    pub fn get_mut(&mut self, key: &str) -> Result<&mut u8, RegisterError> {
        self.registers
            .get_mut(key)
            .ok_or_else(|| RegisterError::InvalidAddress(key.to_owned()))
    }
}

#[derive(Error, Debug)]
pub(crate) enum RomError {
    #[error("loading rom failed {0}")]
    IoError(#[from] io::Error),

    #[error("out of memory {rom_size:?} > {ram_size:?}")]
    OutOfMemory { rom_size: usize, ram_size: usize },
}

pub(crate) struct Rom {
    data: Vec<u8>,
}

impl Rom {
    pub fn load(path: &str) -> Result<Self, RomError> {
        let mut file = File::open(path)?;
        let mut data = vec![];

        file.read_to_end(&mut data)?;

        if data.len() > constants::AVAILABLE_RAM {
            Err(RomError::OutOfMemory {
                rom_size: data.len(),
                ram_size: constants::AVAILABLE_RAM,
            })?
        }

        Ok(Self { data })
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

#[derive(Error, Debug)]
#[error("stack is empty")]
pub struct StackEmptyError;

#[derive(Default)]
pub struct AddressStack(Vec<u16>);

impl AddressStack {
    pub fn pop<T: From<u16>>(&mut self) -> Result<T, StackEmptyError> {
        self.0.pop().ok_or(StackEmptyError).map(|val| T::from(val))
    }

    pub fn push<T: Into<u16>>(&mut self, val: T) {
        self.0.push(val.into());
    }
}
