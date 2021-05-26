//! Trap Handler

use crate::*;

// Modules
pub mod context;
pub mod frame;
pub mod handler;
pub mod raw;

pub use context::InterruptContext;
pub use context::InterruptType;

pub use frame::TrapFrame;

/// Initialize the trap frame into mscratch
pub fn init_trap_frame()
{
    // Initialize the trap frame
    let trap_frame = TrapFrame::new(2);

    // Allocate the stack frame on the kernel heap
    let addr = Box::leak(Box::new(trap_frame)) as *mut TrapFrame as usize;

    // Write the stack frame into the mscratch register
    riscv::register::mscratch::write(addr);
}