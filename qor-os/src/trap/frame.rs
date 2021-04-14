use crate::*;

static STACK_SIZE: usize = 1;

#[repr(C)]
/// Structure to store the information required for the trap frames
#[derive(Debug)]
pub struct TrapFrame
{
    pub regs: [usize; 32],
    pub fregs: [usize; 32],
    pub satp: usize,
    pub trap_stack: *mut u8,
    pub hartid: usize
}

impl TrapFrame
{
    /// Create a new zeroed trap frame (with a newly allocated Stack)
    pub fn zeroed() -> Self
    {
        Self
        {
            regs: [0; 32],
            fregs: [0; 32],
            satp: 0,
            trap_stack: unsafe { crate::mem::kpalloc(STACK_SIZE).add(4096 * STACK_SIZE) },
            hartid: 0
        }
    }
}

/// Initialize the kernel trap frame
pub fn init_trap_frame()
{
    let trap_frame_location = mem::kpalloc(1);
    riscv::register::mscratch::write(trap_frame_location as usize);

    // Safety: This was space allocated by the kernel for this purpose
    unsafe { *(trap_frame_location as *mut trap::TrapFrame) = trap::TrapFrame::zeroed() };
}