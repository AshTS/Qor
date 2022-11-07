//! Handles the assembly files

use core::arch::global_asm;

global_asm!(include_str!("boot.s"));
global_asm!(include_str!("trap.s"));
global_asm!(include_str!("mem.s"));
global_asm!(include_str!("init.s"));

// Values defined in assembly which now need to be brought into rust
extern "C" {
    pub static HEAP_START: usize;
    pub static HEAP_END: usize;
}

// Functions defined in assembly which now need to be brought into rust
extern "C" {
    pub fn init_proc_location();
    pub fn switch_to_user(frame: usize, mepc: usize, satp: usize) -> !;
}
