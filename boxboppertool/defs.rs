// boxboppertool Copyright 2020-2021 David Atkinson
//
// defs.rs: defaults for boxboppertool

pub const DEF_MAX_MOVES: u16 = 200;
pub const DEF_MAX_DEPTH: u16 = 100;
pub const DEF_WIDTH: usize = 5;
pub const DEF_HEIGHT: usize = 5;
pub const DEF_BOX_DENSITY: u32 = 20;
pub const DEF_WALL_DENSITY: u32 = 20;
pub const DEF_VERBOSITY: u32 = 1;
pub const DEF_MAX_MAPS: usize = 4_000_000;  // typically up to 12gig of ram, for 16gig desktop
pub const DEF_MAX_LEVEL: usize = 50;        // maximum level number to check when doing speed test, should be less than BUILTIN_LEVELS.len() 
