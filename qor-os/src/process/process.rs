use crate::*;

// Static PID counter
static PID: core::sync::atomic::AtomicU16 = core::sync::atomic::AtomicU16::new(0);

// Defaults for a process
static DEFAULT_PROCESS_STACK: usize = 4;
static DEFAULT_STACK_ADDR: usize = 0x2000_0000;
static DEFAULT_ENTRY_POINT: usize = 0x4000_0000;

// Bring in assembly function
extern "C"
{
    /// Switch over into user mode
	fn switch_to_user(frame: usize, mepc: usize, satp: usize) -> !;
}
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
#[derive(Debug)]
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

    /// Map a pointer through the page table
    pub fn map_ptr(&mut self, ptr: usize) -> usize
    {
        mem::mmu::inner_virt_to_phys(unsafe { self.root.as_mut() }.unwrap(), ptr).unwrap()
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
            pc: entry_point + func_addr % mem::pages::PAGE_SIZE,
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
        for i in 0..16
        {
            let addr = i * mem::pages::PAGE_SIZE;

            mem::mmu::inner_map(root, entry_point + addr, func_addr + addr,
                            mem::EntryBits::UserReadExecute, mem::mmu::MMUPageLevel::Level4KiB);
        }

        result
    }

    /// Switch to this process
    pub fn switch_to_process(&self) -> !
    {
        let frame_addr = self.get_frame_pointer();
        let mepc = self.get_program_counter();
        let satp = self.get_satp();

        kprintln!("Frame Addr: 0x{:x} MEPC: 0x{:x} SATP: 0x{:x}", frame_addr, mepc, satp);

        unsafe { switch_to_user(frame_addr, mepc, satp) }
    }
    

    /// Get the table reference
    pub fn get_table(&self) -> &mem::pagetable::Table
    {
        unsafe { self.root.as_ref().unwrap() }
    }

    /// Create a new process
    pub fn new_elf(table: *mut mem::pagetable::Table, stack_pages: usize, stack_addr: usize, entry_point: usize) -> Self
    {
        let mut result = Self
        {
            frame: trap::TrapFrame::zeroed(),
            stack: mem::kpalloc(stack_pages),
            pid: PID.fetch_add(1, core::sync::atomic::Ordering::SeqCst),
            pc: entry_point,
            root: table,
            state: ProcessState::Waiting,
        };

        result.frame.satp = result.get_satp();

        // Set the stack pointer
        result.frame.regs[2] = stack_addr + mem::pages::PAGE_SIZE * stack_pages;

        result
    }

    /// Get the pid of the given process
    pub fn get_pid(&self) -> u16
    {
        self.pid
    }

    /// Is the process running
    pub fn is_running(&self) -> bool
    {
        self.state == ProcessState::Running
    }

    /// Is the process dead
    pub fn is_dead(&self) -> bool
    {
        self.state == ProcessState::Dead
    }

    /// Get the frame pointer
    pub fn get_frame_pointer(&self) -> usize
    {
        &self.frame as *const trap::TrapFrame as usize
    }

    /// Get the program counter
    pub fn get_program_counter(&self) -> usize
    {
        self.pc
    }

    /// Get the satp value
    pub fn get_satp(&self) -> usize
    {
        (self.root as usize >> 12) | (8usize << 60) | ((self.pid as usize) << 44)
    }

    /// Start the process
    pub fn start(&mut self)
    {
        self.state = ProcessState::Running;
    }

    /// Update the program counter
    pub fn update_program_counter(&mut self, pc: usize)
    {
        self.pc = pc;
    } 

    /// Halt the process (switch to the Dead state)
    pub fn halt(&mut self)
    {
        self.state = ProcessState::Dead
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