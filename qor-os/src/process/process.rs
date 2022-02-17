use crate::*;
use crate::drivers::timer::KernelTime;
use crate::fs::fstrait::Filesystem;

use fs::structures::DirectoryEntry;
use libutils::paths::PathBuffer;

use super::data::ProcessData;
use super::descriptor::FileDescriptor;
use super::stats::MemoryStats;

use mem::mmu::PageTable;
use mem::mmu::TranslationError;

use super::signals::*;

use trap::TrapFrame;

use super::PID;

// Global PID counter
static mut NEXT_PID: PID = 0;

/// Get the next PID
fn next_pid() -> PID
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

const MAP_ANON: usize = 1;

/// Reasons for a process to be waiting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitMode
{
    // Pointer to return code
    ForChild,
    ForSignal,
    ForIO((usize, usize, *mut u8))
}

/// Process State Enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState
{
    Running,
    Sleeping{wake_time: KernelTime},
    Stopped,
    Waiting(WaitMode),
    Dead,
    Zombie
}
/// Process Structure
#[repr(C)]
pub struct Process
{
    pub frame: *mut TrapFrame,
    pub backup_frame: *mut TrapFrame,
    pub stack: *mut u8,
    pub program_counter: usize,
    pub backup_program_counter: usize,
    pub pid: PID,
    pub root: *mut PageTable,
    pub state: ProcessState,
    pub data: ProcessData,
    pub fs_interface: Option<&'static mut fs::vfs::FilesystemInterface>,
    pub signals: [Option<POSIXSignal>; 4],
    pub exit_code: u32,
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

        let text = mem::lds::text_end() - mem::lds::text_start();
        let data = mem::lds::rodata_end() - mem::lds::rodata_start();

        let mem_stats = MemoryStats::new(0, 0, text / mem::PAGE_SIZE, data / mem::PAGE_SIZE + stack_size);

        Self::from_components(entry_point, page_table_ptr, stack_size, stack, mem_stats)
    }

    /// Create a new process from components
    pub fn from_components(entry_point: usize, page_table: *mut PageTable, stack_size: usize, stack_ptr: usize, mem_stats: MemoryStats) -> Self
    {
        let frame = mem::kpalloc(1, "Trap Frame").unwrap() as *mut TrapFrame;
        let backup_frame = mem::kpalloc(1, "Backup Trap Frame").unwrap() as *mut TrapFrame;

        unsafe { frame.write(TrapFrame::new(4)) }
        unsafe { backup_frame.write(TrapFrame::new(4)) }

        let pid = next_pid();

        // Create the process
        let temp_result = 
            Process
            {
                frame,
                backup_frame,
                stack: stack_ptr as *mut u8,
                program_counter: entry_point,
                backup_program_counter: 0,
                pid: pid,
                root: page_table,
                state: ProcessState::Running,
                data: unsafe { ProcessData::new(stack_size, mem_stats, pid) },
                fs_interface: None,
                signals: [None, None, None, None],
                exit_code: 0
            };

        // Update the stack pointer
        unsafe { temp_result.frame.as_mut().unwrap() }.regs[2] = stack_ptr + stack_size * mem::PAGE_SIZE;

        temp_result
    }

    /// Set the environment arguments
    pub fn set_environment(&mut self, envp: &[&[u8]])
    {
        let mut envp_addrs = Vec::with_capacity(envp.len());
        
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
    }

    /// Set the command line and environment arguments
    pub fn set_arguments(&mut self, args: &[String], envp: &[String])
    {
        // Set argc
        unsafe { self.frame.as_mut().unwrap() }.regs[10] = args.len();

        let mut args_to_store = Vec::new();
        
        let mut arg_addrs = Vec::with_capacity(args.len());
        let mut envp_addrs = Vec::with_capacity(envp.len());

        // Write the arguments
        for s in args
        {
            args_to_store.push(s.clone());
            arg_addrs.push(self.push_buffer(s.as_bytes()));
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
            envp_addrs.push(self.push_buffer(s.as_bytes()));
            self.push_buffer(&[0]);
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
        self.exit_code = value as u32;
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
                    return Ok(errno::EEXIST);
                }

                inode_result
            }
            else
            {
                if (mode & O_CREAT) == 0
                {
                    return Ok(errno::ENOENT);
                }

                let (path, name) = path.split_last();

                let dest_inode = vfs.path_to_inode(&path)?;

                vfs.create_file(dest_inode, name.to_string())?
            };

        let fd = vfs.open_fd(inode, mode)?;
        Ok(self.add_descriptor(fd))
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
            errno::EBADF
        }
    }

    /// Check for data available on a file descriptor
    pub fn check_available(&mut self, fd: usize) -> bool
    {
        self.ensure_fs();

        if let Some(fd) = self.data.descriptors.get_mut(&fd)
        {
            fd.borrow_mut().check_available()
        }
        else
        {
            false
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
            errno::EBADF
        }
    }

    /// Close a file descriptor
    pub fn close(&mut self, fd_number: usize) -> usize
    {
        let v = if let Some(fd) = self.data.descriptors.get_mut(&fd_number)
        {
            fd.borrow_mut().close(self.fs_interface.as_mut().unwrap());
            0
        }
        else
        {
            errno::EBADF // Bad file descriptor
        };

        if v == 0
        {
            self.data.descriptors.remove(&fd_number);
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
            errno::EBADF // Bad file descriptor
        }
    }

    /// Run an ioctl command
    pub fn exec_ioctl(&mut self, fd: usize, cmd: fs::ioctl::IOControlCommand) -> usize
    {
        self.ensure_fs();

        if let Some(fd) = self.data.descriptors.get_mut(&fd)
        {
            if let Some(inode) = fd.borrow_mut().get_inode()
            {
                if let Ok(val) = self.fs_interface.as_mut().unwrap().exec_ioctl(inode, cmd)
                {
                    val
                }
                else
                {
                    usize::MAX
                }
            }
            else
            {
                usize::MAX
            }
        }
        else
        {
            errno::EBADF // Bad File descriptor
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

        let mut temp = Self::from_components(self.program_counter + 4, unsafe { self.root.as_mut().unwrap().duplicate_map() }, stack_size, self.stack as usize, self.data.mem_stats);

        let new_frame = mem::kpalloc(1, "Trap Frame").unwrap() as *mut TrapFrame;

        unsafe { new_frame.write(self.frame.read()) }

        temp.frame = new_frame;
        unsafe { temp.frame.as_mut().unwrap() }.regs[10] = 0;

        temp.data.descriptors = self.data.descriptors.clone();
        
        temp.data.cwd = self.data.cwd.clone();

        temp.data.cmdline_args = self.data.cmdline_args.clone();

        temp.data.process_group_id = self.data.process_group_id;

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
    pub fn register_child(&mut self, child_pid: PID)
    {
        self.data.register_child(child_pid);
    }

    /// Remove a child  process
    pub fn remove_child(&mut self, child_pid: PID)
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
    pub fn get_children(&self) -> &Vec<PID>
    {
        &self.data.children
    }

    /// Map a region of memory with the given permissions
    pub fn map(&mut self, length: usize, perm: mem::mmu::PageTableEntryFlags, flags: usize, fd: usize, offset: usize) -> usize
    {
        if offset != 0
        {
            todo!()
        }

        // Allocate the memory
        let mut ptr_op = None;

        if flags & MAP_ANON == 0 && (flags as i64) >= 0
        {
            if let Some(fd_obj) = self.data.descriptors.get_mut(&fd)
            {
                if let Some(b) = fd_obj.borrow().get_buffer()
                {
                    ptr_op = Some(b);
                }
            }
        }

        let ptr;

        if let Some(p) = ptr_op
        {
            ptr = p as usize;
        }
        else
        {
            ptr = mem::kpzalloc(length, "mmap").unwrap();
        }

        self.data.mem_stats.resident += length;

        let user_addr = self.data.next_heap;

        self.data.mem.push((ptr as *mut u8, length));

        // Map the memory
        for i in 0..length
        {
            unsafe { self.root.as_mut().unwrap() }.map(self.data.next_heap, ptr + i * mem::PAGE_SIZE, perm, 0);
            self.data.next_heap += mem::PAGE_SIZE;
        }

        // If need be, fill the memory
        if flags & MAP_ANON == 0 && (flags as i64) >= 0
        {
            if let Some(fd_obj) = self.data.descriptors.get_mut(&fd)
            {
                if ptr_op.is_none()
                {
                    fd_obj.borrow_mut().seek(offset, process::descriptor::SeekMode::SeekSet);
                    fd_obj.borrow_mut().read(self.fs_interface.as_mut().unwrap(), ptr as *mut u8, 4096 * length);
                }
                
                self.data.mmapped_files.insert(ptr as *mut u8, fd);
            }
            else
            {
                kwarnln!("Bad fd {}", fd);
                return errno::EBADF; // Bad file descriptor
            }
        }

        user_addr
    }

    /// Unmap a region of memory
    pub fn unmap(&mut self, addr: usize, length: usize) -> usize
    {
        // Convert the user address to a physical address
        let phys_addr = self.map_mem(addr).unwrap();

        let mut should_free = true;

        // Check if the mapped region is a file
        if let Some(fd) = self.data.mmapped_files.get(&(phys_addr as *mut u8))
        {
            // TODO: Support offsets

            // If the file still exists, update the cache
            if let Some(fd_obj) = self.data.descriptors.get_mut(&fd)
            {
                if fd_obj.borrow().get_buffer().is_some()
                {
                    should_free = false;
                }
                else
                {
                    fd_obj.borrow_mut().seek(0, process::descriptor::SeekMode::SeekSet);
                    fd_obj.borrow_mut().write(self.fs_interface.as_mut().unwrap(), phys_addr as *mut u8, 4096 * length);
                }
            }

            // And remove the mapping from the process
            self.data.mmapped_files.remove(&(phys_addr as *mut u8));
        }

        // Unmap the memory
        for i in 0..(length / mem::PAGE_SIZE)
        {
            unsafe { self.root.as_mut().unwrap() }.unmap(addr + i * mem::PAGE_SIZE, 0);
        }

        // Free the memory
        if should_free
        {
            mem::kpfree(phys_addr, length / mem::PAGE_SIZE).unwrap();
        }

        // Remove the mapping entry
        let mut index = None;
        for (i, mapping) in self.data.mem.iter().enumerate()
        {
            if mapping.0 as usize == phys_addr
            {
                index = Some(i);
                break;
            }
        }

        if let Some(index) = index
        {
            self.data.mem.remove(index);
        }

        0
    }

    /// Get directory entries for the given file descriptor
    pub fn get_dir_entries(&mut self, fd: usize) -> Result<Vec<DirectoryEntry>, usize>
    {
        let inode = if let Some(desc) = self.data.descriptors.get_mut(&fd)
        {
            if let Some(inode) = desc.borrow_mut().get_inode()
            {
                inode
            }
            else
            {
                return Err(errno::ENOENT); // File not found
            }
        }
        else
        {
            return Err(errno::EBADF); // Bad file descriptor
        };

        self.ensure_fs();

        Ok(self.fs_interface.as_mut().unwrap().get_dir_entries(inode).unwrap())
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

    /// Get the disposition for a given signal
    pub fn get_disposition_for_signal(&mut self, signal: SignalType) -> SignalDisposition
    {
        *self.data.signal_map.get(&signal).unwrap()
    }

    /// Execute the handler for a signal, returns true if the process needs to
    /// be scheduled
    pub fn trigger_signal(&mut self, signal: POSIXSignal) -> bool
    {
        kdebug!(Signals, "PID {} got Signal {:?}, ", self.pid, signal.sig_type);

        match self.get_disposition_for_signal(signal.sig_type)
        {
            SignalDisposition::Terminate =>
            {
                kdebugln!(Signals, "Terminating");
                self.kill(128 + signal.sig_type as usize)
            },
            SignalDisposition::Ignore =>
            { 
                kdebugln!(Signals, "Ignoring");
            },
            SignalDisposition::Handler(addr) => 
            {
                self.switch_to_signal_handler(addr, signal);
                return true;
            },
            SignalDisposition::Core => todo!(),
            SignalDisposition::Stop =>
            {
                kdebugln!(Signals, "Stopping");
                self.state = ProcessState::Stopped;
            },
            SignalDisposition::Continue => 
            {
                kdebugln!(Signals, "Continuing");

                if self.state == ProcessState::Stopped
                {
                    self.state = ProcessState::Running;
                }
            },
        }

        false
    }

    /// Push a signal to the signal stack
    pub fn push_signal(&mut self, signal: POSIXSignal) -> Result<(), ()>
    {
        for val in self.signals.iter_mut()
        {
            if val.is_none()
            {
                *val = Some(signal);
                return Ok(())
            }
        }

        Err(())
    }

    /// Pop a signal from the signal stack
    pub fn pop_signal(&mut self) -> Option<POSIXSignal>
    {
        let result = self.signals[0];

        for i in 1..self.signals.len()
        {
            self.signals[i - 1] = self.signals[i];
        }

        result
    }

    /// Swap out the trap frames
    pub fn swap_frames(&mut self)
    {
        core::mem::swap(
            unsafe { self.frame.as_mut().unwrap() }, 
            unsafe { self.backup_frame.as_mut().unwrap() });

        core::mem::swap(
            &mut self.program_counter,
            &mut self.backup_program_counter);
    }

    /// Return from a signal handler
    pub fn return_from_signal(&mut self)
    {
        kdebugln!(Signals, "Returning from signal on PID {}", self.pid);
        
        self.swap_frames();

        trap::handler::switch_process();
    }

    /// Swap to a signal handler
    pub fn switch_to_signal_handler(&mut self, addr: usize, signal: POSIXSignal)
    {
        self.swap_frames();

        self.program_counter = addr;
        
        if let Some(frame) = unsafe { self.frame.as_mut() }
        {
            frame.regs[10] = signal.sig_type as u32 as usize;
            frame.regs[2] = unsafe { self.backup_frame.as_mut() }.unwrap().regs[2];

            // TODO: Add the signal info structure
        }
        else
        {
            panic!("Process PID {} has an invalid trap frame loaded!", self.pid);
        }
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

        // Drop the backup trap frame
        mem::kpfree(self.backup_frame as usize, 1).unwrap();
    }
}