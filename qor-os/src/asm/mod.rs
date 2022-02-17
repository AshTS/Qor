//! Handles the assembly files

use core::arch::global_asm;

global_asm!(include_str!("boot.s"));
global_asm!(include_str!("trap.s"));
global_asm!(include_str!("mem.s"));
