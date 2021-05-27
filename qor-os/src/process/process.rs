use crate::*;

use super::data::ProcessData;

use mem::mmu::PageTable;

use trap::TrapFrame;

// Global PID counter
static mut NEXT_PID: u16 = 0;

/// Get the next PID
fn next_pid() -> u16
{
    unsafe
    {
        NEXT_PID += 1;
        NEXT_PID - 1
    }
}

/// Process State Enumeration
#[derive(Debug, Clone, Copy)]
pub enum ProcessState
{
    Running,
    Sleeping,
    Waiting,
    Dead
}

/// Process Structure
#[repr(C)]
pub struct Process
{
    pub frame: TrapFrame,
    stack: *mut u8,
    pub program_counter: usize,
    pub pid: u16,
    pub root: *mut PageTable,
    state: ProcessState,
    data: ProcessData
} 

impl Process
{
    /// Create a new process from a function pointer (for testing only)
    pub fn from_fn_ptr(f: fn()) -> Self
    {
        let stack_size = 2;
        let entry_point = f as usize;

        let mut temp_result = 
                Process
                {
                    frame: TrapFrame::new(2),
                    stack: 0 as *mut u8,
                    program_counter: entry_point,
                    pid: next_pid(),
                    root: mem::kpzalloc(1).unwrap() as *mut PageTable,
                    state: ProcessState::Running,
                    data: ProcessData { stack_size, mem_ptr: 0 as *mut u8, mem_size: 0 }
                };

        // Initialize the stack
        let stack = mem::kpzalloc(stack_size).unwrap();
        temp_result.stack = stack as *mut u8;
        temp_result.frame.regs[2] = stack + stack_size * mem::PAGE_SIZE;
        let page_table = unsafe {temp_result.root.as_mut()}.unwrap();

        use mem::mmu::PageTableEntryFlags;

        // Map the Kernel
        page_table.identity_map(mem::lds::text_start(), mem::lds::text_end(), PageTableEntryFlags::readable() | PageTableEntryFlags::executable() | PageTableEntryFlags::user());
        page_table.identity_map(mem::lds::rodata_start(), mem::lds::rodata_end(), PageTableEntryFlags::readable() | PageTableEntryFlags::executable() | PageTableEntryFlags::user());

        // Map the stack
        page_table.identity_map(stack, stack + (stack_size - 1) * mem::PAGE_SIZE, PageTableEntryFlags::readable() | PageTableEntryFlags::writable() | PageTableEntryFlags::user());

        temp_result
    }

    /// Get the current state
    pub fn get_state(&self) -> ProcessState
    {
        self.state
    }

    /// Kill a process
    pub fn kill(&mut self, value: usize)
    {
        kdebugln!(Processes, "Killing PID {} with exit code: {}", self.pid, value);

        self.state = ProcessState::Dead;
    }
}

impl core::ops::Drop for Process
{
    fn drop(&mut self) 
    {


        // Drop the stack
        mem::kpfree(self.stack as usize, self.data.stack_size).unwrap();

        // Drop the page table
        unsafe { self.root.as_mut() }.unwrap().drop_table();

        // Drop the memory allocated to the process
        if !self.data.mem_ptr.is_null()
        {
            mem::kpfree(self.data.mem_ptr as usize, self.data.mem_size).unwrap();
        }
    }
}