use crate::*;
use crate::fs::fstrait::Filesystem;

use fs::structures::DirectoryEntry;
use libutils::paths::PathBuffer;

use super::data::ProcessData;
use super::descriptor::FileDescriptor;

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

// Must be kept in sync with syscalls.h
const O_RDONLY: usize = 1;
const O_WRONLY: usize = 2;
const O_APPEND: usize = 4;
const O_TRUNC: usize =  8;
const O_CREAT: usize =  16;
const O_EXCL: usize =   32;

const SEEK_SET: usize = 1;
const SEEK_CUR: usize = 2;
const SEEK_END: usize = 4;


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
    pub frame: *mut TrapFrame,
    pub stack: *mut u8,
    pub program_counter: usize,
    pub pid: u16,
    pub root: *mut PageTable,
    pub state: ProcessState,
    pub data: ProcessData,
    pub fs_interface: Option<&'static mut fs::vfs::FilesystemInterface>
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
        let frame = mem::kpalloc(1, "Trap Frame").unwrap() as *mut TrapFrame;
        unsafe { frame.write(TrapFrame::new(4)) }

        // Create the process
        let temp_result = 
            Process
            {
                frame,
                stack: stack_ptr as *mut u8,
                program_counter: entry_point,
                pid: next_pid(),
                root: page_table,
                state: ProcessState::Running,
                data: unsafe { ProcessData::new(stack_size) },
                fs_interface: None,
            };

        // Update the stack pointer
        unsafe { temp_result.frame.as_mut().unwrap() }.regs[2] = stack_ptr + stack_size * mem::PAGE_SIZE;

        temp_result
    }

    /// Set the command line and environment arguments
    pub fn set_arguments(&mut self, args: &[&[u8]], envp: &[&[u8]])
    {
        // Set argc
        unsafe { self.frame.as_mut().unwrap() }.regs[10] = args.len();

        let mut args_to_store = Vec::new();
        
        let mut arg_addrs = Vec::with_capacity(args.len());
        let mut envp_addrs = Vec::with_capacity(envp.len());

        // Write the arguments
        for s in args
        {
            args_to_store.push(unsafe { String::from_utf8_unchecked(Vec::from(*s)) });
            arg_addrs.push(self.push_buffer(s));
        }

        // Write the argument array
        let mut ptr = self.push(0usize);
        for v in arg_addrs.iter().rev()
        {
            ptr = self.push(*v);
        }

        // Set argv
        unsafe { self.frame.as_mut().unwrap() }.regs[11] = ptr;

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
        unsafe { self.frame.as_mut().unwrap() }.regs[12] = ptr;

        // Store the arguments in the process data
        self.data.fill_command_line_args(args_to_store);
    }

    /// Push a buffer
    pub fn push_buffer(&mut self, data: &[u8]) -> usize
    {
        // Move the stack pointer down
        unsafe { self.frame.as_mut().unwrap() }.regs[2] -= data.len();

        // Get the physical location of where the buffer must go
        let true_ptr = self.map_mem(unsafe { self.frame.as_mut().unwrap() }.regs[2]).unwrap() as *mut u8;

        // Write to the buffer
        for (i, v) in data.iter().enumerate()
        {
            unsafe { true_ptr.add(i).write(*v) };
        }

        // Return the virtual address of the buffer
        unsafe { self.frame.as_mut().unwrap() }.regs[2]
    }

    /// Push data to the stack
    pub fn push<T>(&mut self, data: T) -> usize
    {
        // Move the stack pointer down
        unsafe { self.frame.as_mut().unwrap() }.regs[2] -= core::mem::size_of::<T>();

        // Set the proper alignment
        let align = core::mem::align_of::<T>();
        unsafe { self.frame.as_mut().unwrap() }.regs[2] &= !(align - 1);

        // Get the physical location of where the buffer must go
        let true_ptr = self.map_mem(unsafe { self.frame.as_mut().unwrap() }.regs[2]).unwrap() as *mut T;

        unsafe { true_ptr.write(data) };

        // Return the virtual address of the buffer
        unsafe { self.frame.as_mut().unwrap() }.regs[2]
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
        self.fs_interface = crate::fs::vfs::get_vfs_reference();
    }

    /// Ensure file system
    pub fn ensure_fs(&mut self)
    {
        if self.fs_interface.is_none()
        {
            self.init_fs();
        }
    }

    /// Write descriptor into the next open file descriptor
    pub fn add_descriptor(&mut self, fd: Box<dyn FileDescriptor>) -> usize
    {
        let mut i = 0;

        while self.data.descriptors.contains_key(&i)
        {
            i += 1;
        }

        self.data.descriptors.insert(i, alloc::sync::Arc::new(core::cell::RefCell::new(fd)));

        i
    }

    /// Open a file by path
    pub fn open(&mut self, path: PathBuffer, mode: usize) -> Result<usize, fs::structures::FilesystemError>
    {
        self.ensure_fs();

        let vfs = self.fs_interface.as_mut().unwrap();
        let inode = 
            if let Ok(inode_result) = vfs.path_to_inode(&path)
            {
                if (mode & O_EXCL) > 0
                {
                    return Ok(usize::MAX);
                }

                inode_result
            }
            else
            {
                if (mode & O_CREAT) == 0
                {
                    return Ok(usize::MAX);
                }

                let (path, name) = path.split_last();

                let dest_inode = vfs.path_to_inode(&path)?;

                vfs.create_file(dest_inode, name.to_string())?
            };

        if let Ok(fd) = vfs.open_fd(inode, mode)
        {
            Ok(self.add_descriptor(fd))
        }
        else
        {
            Ok(usize::MAX)
        }
    }

    /// Read from a file descriptor
    pub fn read(&mut self, fd: usize, buffer: *mut u8, count: usize) -> usize
    {
        self.ensure_fs();

        if let Some(fd) = self.data.descriptors.get_mut(&fd)
        {
            fd.borrow_mut().read(self.fs_interface.as_mut().unwrap(), buffer, count)
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
            fd.borrow_mut().write(self.fs_interface.as_mut().unwrap(), buffer, count)
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
            fd.borrow_mut().close(self.fs_interface.as_mut().unwrap());
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

    /// Create a new pipe
    pub fn pipe(&mut self) -> (usize, usize)
    {
        let (read, write) = super::pipe::new_pipe();

        let read = self.add_descriptor(Box::new(read));
        let write = self.add_descriptor(Box::new(write));

        (read, write)
    }

    /// Duplicate a file descriptor
    pub fn dup(&mut self, old: usize, new: Option<usize>) -> usize
    {
        let fd = if let Some(fd) = self.data.descriptors.get(&old)
        {
            fd.clone()
        }
        else
        {
            return usize::MAX;
        };

        let out = if let Some(new) = new
        {
            new
        }
        else
        {
            let mut i = 0;

            while self.data.descriptors.contains_key(&i)
            {
                i += 1;
            }

            i
        };

        self.data.descriptors.insert(out, fd);

        if new.is_some()
        {
            0
        }
        else
        {
            out
        }
    }

    /// Seek to a location in the file descriptor
    pub fn seek(&mut self, fd: usize, offset: usize, mode: usize) -> usize
    {
        use super::descriptor::SeekMode;

        let enum_mode = match mode
        {
            SEEK_CUR => SeekMode::SeekCurrent,
            SEEK_END => SeekMode::SeekEnd,
            SEEK_SET => SeekMode::SeekSet,
            _ => { return offset.wrapping_sub(1); }
        };

        if let Some(fd) = self.data.descriptors.get_mut(&fd)
        {
            fd.borrow_mut().seek(offset, enum_mode)
        }
        else
        {
            usize::MAX
        }
    }

    /// Display the memory map for this process
    pub fn display_memory_map(&self)
    {
        let pt = unsafe { self.root.as_ref().unwrap() };

        pt.display_mapping();
    }

    /// Get a forked version of the current process
    pub fn forked(&mut self) -> Self
    {
        let stack_size = self.data.stack_size;

        let mut temp = Self::from_components(self.program_counter + 4, unsafe { self.root.as_mut().unwrap().duplicate_map() }, stack_size, self.stack as usize);

        let new_frame = mem::kpalloc(1, "Trap Frame").unwrap() as *mut TrapFrame;

        unsafe { new_frame.write(self.frame.read()) }

        temp.frame = new_frame;
        unsafe { temp.frame.as_mut().unwrap() }.regs[10] = 0;

        temp.data.descriptors = self.data.descriptors.clone();
        
        temp.data.cwd = self.data.cwd.clone();

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
        let ptr = mem::kpzalloc(length, "mmap").unwrap();

        let user_addr = self.data.next_heap;

        self.data.mem.push((ptr as *mut u8, length));

        // Map the memory
        for i in 0..length
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

    /// Report a fatal fault in the process
    pub fn report_fault(&mut self, fault_msg: &str)
    {
        kerrorln!("Process PID {} Encountered a Fatal Fault: ", self.pid);
        kerrorln!("   {}", fault_msg);

        self.kill(1);
    }

    /// Get directory entries for the given file descriptor
    pub fn get_dir_entries(&mut self, fd: usize) -> Option<Vec<DirectoryEntry>>
    {
        let inode = if let Some(desc) = self.data.descriptors.get_mut(&fd)
        {
            if let Some(inode) = desc.borrow_mut().get_inode()
            {
                inode
            }
            else
            {
                return None;
            }
        }
        else
        {
            return None;
        };

        self.ensure_fs();

        Some(self.fs_interface.as_mut().unwrap().get_dir_entries(inode).unwrap())
    }

    /// Get the total memory held by the process in pages
    pub fn get_process_memory(&self) -> usize
    {
        let mut total = 0;

        for (_, size) in &self.data.mem
        {
            total += size;
        }

        total
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
        
        // Drop the trap frame
        mem::kpfree(self.frame as usize, 1).unwrap();
    }
}