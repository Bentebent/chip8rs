pub const TOTAL_RAM: usize = 0x1000;
pub const INSTRUCTIONS_PER_SECOND: usize = 700;
pub const MS_PER_INSTRUCTION: f64 = 1000.0 / INSTRUCTIONS_PER_SECOND as f64;
pub const MS_60HZ: f64 = 1000.0 / 60.0;
pub const MEMORY_OFFSET: usize = 0x200;
pub const DISPLAY_RANGE: (usize, usize) = (0xF00, 0xFFF);
pub const RAM_RANGE: (usize, usize) = (MEMORY_OFFSET, DISPLAY_RANGE.0);
pub const AVAILABLE_RAM: usize = RAM_RANGE.1 - RAM_RANGE.0;
