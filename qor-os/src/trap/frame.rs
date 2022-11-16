use libutils::sync::NoInterruptMarker;

use crate::{mem::{KernelPageBox, KiByteCount, PageCount, PAGE_SIZE}, process::ProcessIdentifier};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct TrapFrame {
    pub regs: [usize; 32],
    pub fregs: [f64; 32],
    pub satp: usize,
    pub trap_stack: *mut u8,
    pub hartid: usize,
    pub trap_stack_size: usize,
    pub pid: ProcessIdentifier
}

impl TrapFrame {
    /// Create a new trap frame, allocating a stack of the given size
    pub fn new(no_interrupts: NoInterruptMarker, stack_size: KiByteCount) -> Self {
        // Convert the stack over to pages
        let stack_size: PageCount = stack_size.convert();

        // Statically allocate the stack
        let stack =
            match crate::mem::PAGE_ALLOCATOR.allocate_static_pages(no_interrupts, stack_size) {
                Ok(stack) => stack as *mut [[u8; PAGE_SIZE]] as *mut u8,
                Err(e) => panic!("Unable to allocate {} for trap stack: {}", stack_size, e),
            };

        Self {
            regs: [0; 32],
            fregs: [0.0f64; 32],
            satp: 0,
            trap_stack: stack,
            hartid: 0,
            trap_stack_size: stack_size.raw(),
            pid: 0
        }
    }
}

impl core::ops::Drop for TrapFrame {
    fn drop(&mut self) {
        unsafe { KernelPageBox::from_raw(self.trap_stack, self.trap_stack_size, 0) };
    }
}

/// Initialize the global trap frame
pub fn initialize_trap_frame(no_interrupts: NoInterruptMarker) {
    // Construct the new trap frame
    let frame = TrapFrame::new(no_interrupts, PageCount::new(1).convert());

    // Statically allocate the trap frame
    let static_allocated = crate::mem::PAGE_ALLOCATOR
        .allocate_static(no_interrupts, frame)
        .expect("Unable to allocate space for global trap frame");

    // Write that value to the mscratch register
    riscv::register::mscratch::write(static_allocated as *mut _ as usize);
}
