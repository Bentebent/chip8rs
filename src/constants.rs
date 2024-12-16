pub const RAM_SIZE: usize = 0x1000;
pub const INSTRUCTIONS_PER_SECOND: usize = 700;
pub const MS_PER_INSTRUCTION: f64 = 1000. / INSTRUCTIONS_PER_SECOND as f64;
pub const MS_60HZ: f64 = 1. / 60.;
pub const MEMORY_OFFSET: usize = 0x200;
pub const DISPLAY_RANGE: (usize, usize) = (0xF00, 0xFFF);
