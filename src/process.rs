#![allow(non_snake_case)]
use std::num::ParseIntError;

use macroquad::{
    audio::{
        play_sound,
        PlaySoundParams,
        Sound,
    },
    camera::{
        set_camera,
        Camera2D,
    },
    color::{
        self,
        Color,
    },
    shapes::draw_rectangle,
    window::clear_background,
};
use thiserror::Error;

use crate::{
    constants,
    emulator::{
        InstructionData,
        Interpreter,
        KeyPad,
        ProgramCounter,
    },
    mem::{
        AddressStack,
        Ram,
        RamError,
        Register,
        RegisterError,
        StackEmptyError,
    },
};

#[derive(Error, Debug)]
pub(crate) enum ProcessingError {
    #[error("invalid register {source:?}")]
    RegisterError {
        #[from]
        source: RegisterError,
    },

    #[error("invalid hex value {source:?}")]
    HexParseError {
        #[from]
        source: ParseIntError,
    },

    #[error("invalid jump")]
    JumpOutOfBounds {
        #[from]
        source: StackEmptyError,
    },

    #[error("invalid memory address")]
    AddressOutOfBounds {
        #[from]
        source: RamError,
    },
}

pub fn op_00E0(camera: &Camera2D, color: Color, ram: &mut Ram) {
    set_camera(camera);
    clear_background(color);
    ram.reset_vram();
}

pub fn op_00EE(pc: &mut ProgramCounter, stack: &mut AddressStack) -> Result<(), ProcessingError> {
    pc.jump(stack.pop::<usize>()?);
    Ok(())
}

pub fn op_1NNN(pc: &mut ProgramCounter, nnn: u16) {
    pc.jump(nnn);
}

pub fn op_2NNN(stack: &mut AddressStack, pc: &mut ProgramCounter, nnn: u16) {
    stack.push(*pc.inner() as u16);
    pc.jump(nnn);
}

pub fn op_3XNN(register: &Register, x: String, nn: u8, pc: &mut ProgramCounter) -> Result<(), ProcessingError> {
    if register.get(&x)? == nn {
        pc.increment();
    }

    Ok(())
}

pub fn op_4XNN(register: &Register, x: String, nn: u8, pc: &mut ProgramCounter) -> Result<(), ProcessingError> {
    if register.get(&x)? != nn {
        pc.increment();
    }
    Ok(())
}

pub fn op_5XNN(register: &Register, x: String, y: String, pc: &mut ProgramCounter) -> Result<(), ProcessingError> {
    if register.cmp_registers(&x, &y)? {
        pc.increment();
    }
    Ok(())
}

pub fn op_6XNN(register: &mut Register, x: String, nn: u8) -> Result<(), ProcessingError> {
    register.set(&x, nn)?;
    Ok(())
}

pub fn op_7XNN(register: &mut Register, x: String, nn: u8) -> Result<(), ProcessingError> {
    register.set(&x, register.get(&x)?.wrapping_add(nn))?;
    Ok(())
}

pub fn op_8XY0(register: &mut Register, x: String, y: String) -> Result<(), ProcessingError> {
    register.set_x_y(&x, &y)?;
    Ok(())
}

pub fn op_8XY1(register: &mut Register, x: String, y: String) -> Result<(), ProcessingError> {
    *register.get_mut(&x)? |= register.get(&y)?;
    Ok(())
}
pub fn op_8XY2(register: &mut Register, x: String, y: String) -> Result<(), ProcessingError> {
    *register.get_mut(&x)? &= register.get(&y)?;
    Ok(())
}
pub fn op_8XY3(register: &mut Register, x: String, y: String) -> Result<(), ProcessingError> {
    *register.get_mut(&x)? ^= register.get(&y)?;
    Ok(())
}
pub fn op_8XY4(register: &mut Register, x: String, y: String) -> Result<(), ProcessingError> {
    let (val, overflow) = register.get(&x)?.overflowing_add(register.get(&y)?);
    register.set(&x, val)?;
    register.set("VF", overflow as u8)?;
    Ok(())
}
pub fn op_8XY5(register: &mut Register, x: String, y: String) -> Result<(), ProcessingError> {
    let (val, overflow) = register.get(&x)?.overflowing_sub(register.get(&y)?);
    register.set(&x, val)?;
    register.set("VF", !overflow as u8)?;
    Ok(())
}
pub fn op_8XY6(
    interpreter: &Interpreter,
    register: &mut Register,
    x: String,
    y: String,
) -> Result<(), ProcessingError> {
    if let Interpreter::CosmacVIP = interpreter {
        *register.get_mut(&x)? = register.get(&y)?;
    }
    let lsb = register.get(&x)? & 1;
    *register.get_mut(&x)? >>= 1;
    register.set("VF", lsb)?;
    Ok(())
}

pub fn op_8XY7(register: &mut Register, x: String, y: String) -> Result<(), ProcessingError> {
    let (val, overflow) = register.get(&y)?.overflowing_sub(register.get(&x)?);
    register.set(&x, val)?;
    register.set("VF", !overflow as u8)?;
    Ok(())
}

pub fn op_8XYE(
    interpreter: &Interpreter,
    register: &mut Register,
    x: String,
    y: String,
) -> Result<(), ProcessingError> {
    if let Interpreter::CosmacVIP = interpreter {
        register.set_x_y(&x, &y)?;
    }
    let msb = (register.get(&x)? >> 7) & 1;
    *register.get_mut(&x)? <<= 1;
    register.set("VF", msb)?;
    Ok(())
}

pub fn op_9XY0(register: &Register, x: String, y: String, pc: &mut ProgramCounter) -> Result<(), ProcessingError> {
    if !register.cmp_registers(&x, &y)? {
        pc.increment();
    }
    Ok(())
}

pub fn op_ANNN(index_register: &mut u16, nnn: u16) {
    *index_register = nnn;
}

pub fn op_BNNN(
    interpreter: &Interpreter,
    register: &Register,
    pc: &mut ProgramCounter,
    x: String,
    nnn: u16,
) -> Result<(), ProcessingError> {
    match interpreter {
        Interpreter::CosmacVIP => {
            pc.jump(nnn + register.get("V0")? as u16);
        }
        Interpreter::Chip48 | Interpreter::SuperChip => {
            pc.jump(nnn + register.get(&x)? as u16);
        }
    }
    Ok(())
}

pub fn op_CXNN(register: &mut Register, x: String, nn: u8) -> Result<(), ProcessingError> {
    register.set(&x, rand::random::<u8>() & nn)?;
    Ok(())
}

pub fn DXYN(
    memory: &mut Ram,
    register: &mut Register,
    index_register: u16,
    camera: &Camera2D,
    window_size: &(i32, i32),
    pixel_size: i32,
    instruction: InstructionData,
) -> Result<(), ProcessingError> {
    let start_x = (register.get(&instruction.x)? as i32) % window_size.0;
    let start_y = (register.get(&instruction.y)? as i32) % window_size.1;
    register.set("VF", 0)?;

    set_camera(camera);
    let sprite_height = instruction.n;
    let mut bit_flipped_off = false;
    for y_coord in 0..sprite_height {
        let sprite = memory.get(index_register + y_coord)?;
        let screen_pos_y = start_y + y_coord as i32;

        if screen_pos_y >= window_size.1 {
            continue; // Skip rows that exceed the screen height
        }

        for x in 0..8 {
            let screen_pos_x = start_x + (7 - x);
            if screen_pos_x >= window_size.0 {
                continue; // Skip columns that exceed the screen width
            }

            // Get the current pixel in the sprite
            let bit = (sprite >> x) & 1;
            if bit == 0 {
                continue; // Skip processing for pixels that are not set in the sprite
            }

            // Calculate the display bit index and position
            let display_bit_idx =
                (constants::DISPLAY_RANGE.0 as u32 * 8) + (screen_pos_y * window_size.0 + screen_pos_x) as u32;
            let display_byte_idx = display_bit_idx / 8; // 8 bits in a byte
            let display_bit_pos = (display_bit_idx % 8) as u8;

            // Modify the display byte
            let display_byte = memory.get_mut(display_byte_idx as usize)?;
            let display_bit = (*display_byte >> display_bit_pos) & 1;

            if display_bit == 1 {
                bit_flipped_off = true;
            }
            *display_byte ^= 1 << display_bit_pos;

            // Determine the color and draw the pixel
            let color = if (*display_byte >> display_bit_pos) & 1 == 1 {
                color::Color {
                    r: 0.0,
                    g: 128.0,
                    b: 0.0,
                    a: 1.0,
                }
            } else {
                color::BLACK
            };

            draw_rectangle(
                (screen_pos_x * pixel_size) as f32,
                (screen_pos_y * pixel_size) as f32,
                pixel_size as f32,
                pixel_size as f32,
                color,
            );
        }
    }

    // Set VF if a pixel is flipped off
    register.set("VF", bit_flipped_off as u8)?;
    Ok(())
}

pub fn op_EX9E(
    register: &Register,
    keypad: &KeyPad,
    pc: &mut ProgramCounter,
    x: String,
) -> Result<(), ProcessingError> {
    if keypad.is_key_pressed(register.get(&x)?) {
        pc.increment();
    }
    Ok(())
}

pub fn op_EXA1(
    register: &Register,
    keypad: &KeyPad,
    pc: &mut ProgramCounter,
    x: String,
) -> Result<(), ProcessingError> {
    if !keypad.is_key_pressed(register.get(&x)?) {
        pc.increment();
    }

    Ok(())
}
pub fn op_FX07(register: &mut Register, x: String, delay_timer: &u8) -> Result<(), ProcessingError> {
    register.set(&x, *delay_timer)?;
    Ok(())
}

pub fn op_FX15(register: &mut Register, x: String, delay_timer: &mut u8) -> Result<(), ProcessingError> {
    *delay_timer = register.get(&x)?;
    Ok(())
}

pub fn op_FX18(register: &mut Register, x: String, sound_timer: &mut u8, sound: &Sound) -> Result<(), ProcessingError> {
    if *sound_timer == 0 {
        play_sound(
            sound,
            PlaySoundParams {
                looped: true,
                volume: 0.5,
            },
        );
    }
    *sound_timer = register.get(&x)?;
    Ok(())
}

pub fn op_FX1E(register: &Register, x: String, index_register: &mut u16) -> Result<(), ProcessingError> {
    *index_register = index_register.wrapping_add(register.get(&x)? as u16);
    Ok(())
}

pub fn op_FX0A(
    register: &mut Register,
    pc: &mut ProgramCounter,
    keypad: &KeyPad,
    x: String,
) -> Result<(), ProcessingError> {
    if let Some(key_hex) = keypad.get_key_released() {
        register.set(&x, key_hex)?;
    } else {
        pc.decrement();
    }

    Ok(())
}

pub fn op_FX29(register: &Register, index_register: &mut u16, x: String) -> Result<(), ProcessingError> {
    let font_char = register.get(&x)?;
    *index_register = (font_char * 5) as u16;

    Ok(())
}

pub fn op_FX33(register: &Register, memory: &mut Ram, x: String, index_register: u16) -> Result<(), ProcessingError> {
    let mut val = register.get(&x)?;

    for i in (0..3).rev() {
        let remainder = val % 10;
        val /= 10;
        *memory.get_mut(index_register + i)? = remainder;
    }

    Ok(())
}

pub fn op_FX55(
    interpreter: &Interpreter,
    register: &Register,
    memory: &mut Ram,
    index_register: &mut u16,
    x: String,
) -> Result<(), ProcessingError> {
    let range: u16 = u16::from_str_radix(&x[1..], 16)?;
    for i in 0..=range {
        let addr = if let Interpreter::CosmacVIP = interpreter {
            *index_register += i;
            *index_register
        } else {
            *index_register + i
        };
        *memory.get_mut(addr)? = register.get(&format!("V{:X}", i))?;
    }
    Ok(())
}

pub fn op_FX65(
    interpreter: &Interpreter,
    register: &mut Register,
    memory: &Ram,
    index_register: &mut u16,
    x: String,
) -> Result<(), ProcessingError> {
    let range: u16 = u16::from_str_radix(&x[1..], 16)?;
    for i in 0..=range {
        let addr = if let Interpreter::CosmacVIP = interpreter {
            *index_register += i;
            *index_register
        } else {
            *index_register + i
        };
        register.set(&format!("V{:X}", i), memory.get(addr)?)?;
    }
    Ok(())
}
