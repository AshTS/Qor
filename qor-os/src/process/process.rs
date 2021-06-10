use crate::*;

use alloc::format;

use super::data::ProcessData;

use mem::mmu::PageTable;
use mem::mmu::TranslationError;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState
{
    Running,
    Sleeping,
    Waiting,
    Dead,
    Zombie
}
/// Process Structure
#[repr(C)]
pub struct Process
{
    pub frame: TrapFrame,
    pub stack: *mut u8,
    pub program_counter: usize,
    pub pid: u16,
    pub root: *mut PageTable,
    pub state: ProcessState,
    pub data: ProcessData,
    pub fs_interface: Option<Box<fs::interface::FilesystemInterface>>
} 

impl Process
{
    /// Create a new process from a function pointer
    pub fn from_fn_ptr(f: fn()) -> Self
    {
        let stack_size = 2;
        let entry_point = f as usize;

        let page_table_ptr = mem::kpzalloc(1, "Fn Ptr Page Table").unwrap() as *mut PageTable;

        // Initialize the stack
        let stack = mem::kpzalloc(stack_size, "Fn Ptr Stack").unwrap();

        let page_table = unsafe {page_table_ptr.as_mut()}.unwrap();

        use mem::mmu::PageTableEntryFlags;

        // Map the Kernel
        page_table.identity_map(mem::lds::text_start(), mem::lds::text_end(), PageTableEntryFlags::readable() | PageTableEntryFlags::executable() | PageTableEntryFlags::user());
        page_table.identity_map(mem::lds::rodata_start(), mem::lds::rodata_end(), PageTableEntryFlags::readable() | PageTableEntryFlags::executable() | PageTableEntryFlags::user());

        // Map the stack
        page_table.identity_map(stack, stack + (stack_size - 1) * mem::PAGE_SIZE, PageTableEntryFlags::readable() | PageTableEntryFlags::writable() | PageTableEntryFlags::user());

        Self::from_components(entry_point, page_table_ptr, stack_size, stack)
    }

    /// Create a new process from components
    pub fn from_components(entry_point: usize, page_table: *mut PageTable, stack_size: usize, stack_ptr: usize) -> Self
    {
        // Create the process
        let mut temp_result = 
            Process
            {
                frame: TrapFrame::new(4),
                stack: stack_ptr as *mut u8,
                program_counter: entry_point,
                pid: next_pid(),
                root: page_table,
                state: ProcessState::Running,
                data: unsafe { ProcessData::new(stack_size) },
                fs_interface: None,
            };

        // Update the stack pointer
        temp_result.frame.regs[2] = stack_ptr + stack_size * mem::PAGE_SIZE;

        temp_result
    }

    /// Set the command line and environment arguments
    pub fn set_arguments(&mut self, args: &[&[u8]], envp: &[&[u8]])
    {
        // Set argc
        self.frame.regs[10] = args.len();
        
        let mut arg_addrs = Vec::with_capacity(args.len());
        let mut envp_addrs = Vec::with_capacity(envp.len());

        // Write the arguments
        for s in args
        {
            arg_addrs.push(self.push_buffer(s));
        }

        // Write the argument array
        let mut ptr = self.push(0usize);
        for v in arg_addrs.iter().rev()
        {
            ptr = self.push(*v);
        }

        // Set argv
        self.frame.regs[11] = ptr;

        // Write the environment variables
        for s in envp
        {
            envp_addrs.push(self.push_buffer(s));
        }

        // Write envp
        let mut ptr = self.push(0usize);
        for v in envp_addrs.iter().rev()
        {
            ptr = self.push(*v);
        }

        // Set envp
        self.frame.regs[12] = ptr;
    }

    /// Push a buffer
    pub fn push_buffer(&mut self, data: &[u8]) -> usize
    {
        // Move the stack pointer down
        self.frame.regs[2] -= data.len();

        // Get the physical location of where the buffer must go
        let true_ptr = self.map_mem(self.frame.regs[2]).unwrap() as *mut u8;

        // Write to the buffer
        for (i, v) in data.iter().enumerate()
        {
            unsafe { true_ptr.add(i).write(*v) };
        }

        // Return the virtual address of the buffer
        self.frame.regs[2]
    }

    /// Push data to the stack
    pub fn push<T>(&mut self, data: T) -> usize
    {
        // Move the stack pointer down
        self.frame.regs[2] -= core::mem::size_of::<T>();

        // Set the proper alignment
        let align = core::mem::align_of::<T>();
        self.frame.regs[2] &= !(align - 1);

        // Get the physical location of where the buffer must go
        let true_ptr = self.map_mem(self.frame.regs[2]).unwrap() as *mut T;

        unsafe { true_ptr.write(data) };

        // Return the virtual address of the buffer
        self.frame.regs[2]
    }

    /// Map memory based on its page table
    pub fn map_mem(&self, addr: usize) -> Result<usize, TranslationError>
    {
        unsafe { (*self.root).virt_to_phys(addr) }
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

        self.state = ProcessState::Zombie;
    }

    /// Initialize the file system
    pub fn init_fs(&mut self)
    {
        let mut fsi = Box::new(fs::interface::FilesystemInterface::new(0));
        fsi.initialize().unwrap();
        self.fs_interface = Some(fsi);
    }

    /// Ensure file system
    pub fn ensure_fs(&mut self)
    {
        if self.fs_interface.is_none()
        {
            self.init_fs();
        }
    }

    /// Open a file by path
    pub fn open(&mut self, path: &str, _mode: usize) -> Result<usize, fs::interface::FilesystemError>
    {
        self.ensure_fs();
        
        
        let inode = 
            if path.starts_with("/")
            {
                self.fs_interface.as_mut().unwrap().get_inode_by_path(path)?
            }
            else
            {
                let combined = format!("{}{}", self.data.cwd, path);
                self.fs_interface.as_mut().unwrap().get_inode_by_path(&combined)?
            };

        let mut i = 3;

        while self.data.descriptors.contains_key(&i)
        {
            i += 1;
        }

        self.data.descriptors.insert(i, Box::new(super::descriptor::InodeFileDescriptor::new(inode)));

        Ok(i) 
    }

    /// Read from a file descriptor
    pub fn read(&mut self, fd: usize, buffer: *mut u8, count: usize) -> usize
    {
        self.ensure_fs();

        if let Some(fd) = self.data.descriptors.get_mut(&fd)
        {
            fd.read(self.fs_interface.as_mut().unwrap(), buffer, count)
        }
        else
        {
            0xFFFFFFFFFFFFFFFF
        }
    }

    /// Write to a file descriptor
    pub fn write(&mut self, fd: usize, buffer: *mut u8, count: usize) -> usize
    {
        self.ensure_fs();

        if let Some(fd) = self.data.descriptors.get_mut(&fd)
        {
            fd.write(self.fs_interface.as_mut().unwrap(), buffer, count)
        }
        else
        {
            0xFFFFFFFFFFFFFFFF
        }
    }

    /// Close a file descriptor
    pub fn close(&mut self, fd: usize) -> usize
    {
        let v = if let Some(fd) = self.data.descriptors.get_mut(&fd)
        {
            fd.close();
            0
        }
        else
        {
            0xFFFFFFFFFFFFFFFF
        };

        if v == 0
        {
            self.data.descriptors.remove(&fd);
        }

        v
    }

    /// Display the memory map for this process
    pub fn display_memory_map(&self)
    {
        let pt = unsafe { self.root.as_ref().unwrap() };

        pt.display_mapping();
    }

    /// Connect the process to a terminal
    pub fn connect_to_term(&mut self)
    {
        self.data.connect_to_term();
    }

    /// Get a forked version of the current process
    pub fn forked(&mut self) -> Self
    {
        let stack_size = self.data.stack_size;

        let mut temp = Self::from_components(self.program_counter + 4, unsafe { self.root.as_mut().unwrap().duplicate_map() }, stack_size, self.stack as usize);

        temp.frame = self.frame.clone();
        temp.frame.regs[10] = 0;

        temp.connect_to_term();

        self.register_child(temp.pid);

        temp
    }

    /// Check if the state has changed for the wait syscall
    pub fn wait_check(&mut self) -> bool
    {
        if self.state == ProcessState::Zombie
        {
            self.state = ProcessState::Dead;
            true
        }
        else
        {
            false
        }
    }

    /// Register a child with the process
    pub fn register_child(&mut self, child_pid: u16)
    {
        self.data.register_child(child_pid);
    }

    /// Remove a child  process
    pub fn remove_child(&mut self, child_pid: u16)
    {
        for i in 0..self.data.children.len()
        {
            if self.data.children[i] == child_pid
            {
                self.data.children.remove(i);
                break;
            }
        }
    }

    /// Get a reference to the children pids
    pub fn get_children(&self) -> &Vec<u16>
    {
        &self.data.children
    }

    /// Map a region of memory with the given permissions
    pub fn map(&mut self, length: usize, perm: mem::mmu::PageTableEntryFlags) -> usize
    {
        // Allocate the memory
        let ptr = mem::kpzalloc(length / mem::PAGE_SIZE, "mmap").unwrap();

        let user_addr = self.data.next_heap;

        // Map the memory
        for i in 0..(length / mem::PAGE_SIZE)
        {
            unsafe { self.root.as_mut().unwrap() }.map(self.data.next_heap, ptr + i * mem::PAGE_SIZE, perm, 0);
            self.data.next_heap += mem::PAGE_SIZE;
        }

        user_addr
    }

    /// Unmap a region of memory
    pub fn unmap(&mut self, addr: usize, length: usize) -> usize
    {
        // Convert the user address to a physical address
        // TODO: Free physical memory here aswell
        // let phys_addr = self.map_mem(addr).unwrap();

        // Unmap the memory
        for i in 0..(length / mem::PAGE_SIZE)
        {
            unsafe { self.root.as_mut().unwrap() }.unmap(addr + i * mem::PAGE_SIZE, 0);
        }

        self.map_mem(addr).unwrap();

        0
    }

}

impl core::ops::Drop for Process
{
    fn drop(&mut self) 
    {
        for i in 0..self.data.stack_size
        {
            let true_stack = unsafe { (*self.root).virt_to_phys(self.stack as usize + mem::PAGE_SIZE * i) }.unwrap();

            // Drop the stack
            mem::kpfree(true_stack, 1).unwrap();
        }

        // Drop the page table
        unsafe { self.root.as_mut() }.unwrap().drop_table();

        // Drop the memory allocated to the process
        for (ptr, length) in &self.data.mem
        {
            if !ptr.is_null()
            {
                mem::kpfree(*ptr as usize, *length).unwrap();
            }
        }
        
    }
}