use crate::*;

// Static PID counter
static PID: core::sync::atomic::AtomicU16 = core::sync::atomic::AtomicU16::new(0);

// Defaults for a process
static DEFAULT_PROCESS_STACK: usize = 4;
static DEFAULT_STACK_ADDR: usize = 0x1_8000_0000;
static DEFAULT_ENTRY_POINT: usize = 0x1_0000_0000;

/// States a process can be in
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState
{
    Running,
    Sleeping,
    Waiting,
    Dead
}

#[repr(C)]
/// Holds all data required by a process
pub struct ProcessData
{
    frame: trap::TrapFrame,
    stack: *mut u8,
    pc: usize,
    pid: u16,
    root: *mut mem::pagetable::Table,
    state: ProcessState
}

impl ProcessData

{
    /// Instantiate and allocate space for a new default process
    pub fn new_default(func: fn()) -> Self
    {
        Self::new(func, DEFAULT_PROCESS_STACK, DEFAULT_STACK_ADDR, DEFAULT_ENTRY_POINT)
    }

    /// Instantiate and allocate space for a new process 
    pub fn new(func: fn(), stack_pages: usize, stack_addr: usize, entry_point: usize) -> Self
    {
        // Get the physical address of the function
        let func_addr = func as usize;

        // Set a preliminary result
        let mut result = Self
        {
            frame: trap::TrapFrame::zeroed(),
            stack: mem::kpalloc(stack_pages),
            pid: PID.fetch_add(1, core::sync::atomic::Ordering::SeqCst),
            pc: entry_point,
            root: mem::kpzalloc(1) as *mut mem::pagetable::Table,
            state: ProcessState::Waiting,

        };

        // Set the stack pointer
        result.frame.regs[2] = stack_addr + mem::pages::PAGE_SIZE * stack_pages;

        // Get a reference to the root page table
        // Safety: This is memory allocated for this table from the kernel
        let root = unsafe { &mut *result.root };

        // Map the stack
        for i in 0..stack_pages
        {
            let addr = i * mem::pages::PAGE_SIZE;

            mem::mmu::inner_map(root, stack_addr + addr, result.stack as usize + addr,
                            mem::EntryBits::UserReadWrite, mem::mmu::MMUPageLevel::Level4KiB);
        }

        // Map the program space
        for i in 0..2
        {
            let addr = i * mem::pages::PAGE_SIZE;

            mem::mmu::inner_map(root, entry_point + addr, func_addr + addr,
                            mem::EntryBits::UserReadExecute, mem::mmu::MMUPageLevel::Level4KiB);
        }

        unimplemented!()
    }
}

impl core::ops::Drop for ProcessData
{
    fn drop(&mut self)
    {
        // First free the stack
        // Safety: If the stack was properly initialized, this is safe
        unsafe { mem::kpfree(self.stack, 1) };

        // Free the page table
        // Safety: If the page table was properly initialized, this is safe
        unsafe { mem::mmu::unmap_table(self.root) };

        // Free the root of the page table
        // Safety: If the page table was properly initialized, this is safe
        unsafe { mem::kpfree(self.root as *mut u8, 1) };
    }
}