use crate::*;

use trap::TrapFrame;

/// General system call handler
pub fn syscall_handle(mepc: usize, frame: *mut TrapFrame) -> usize
{
    // Extract the syscall number as the first argument
    let syscall_number = unsafe { (*frame).regs[10] };

    kprintln!("Got system call `{}`", syscall_number);

    mepc + 4
}