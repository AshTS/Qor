#[repr(C)]
#[derive(Clone, Copy)]
/// Structure to store the information required for the trap frames
pub struct TrapFrame
{
    pub regs: [usize; 32],
    pub fregs: [usize; 32],
    pub satp: usize,
    pub trap_stack: *mut u8,
    pub hartid: usize
}