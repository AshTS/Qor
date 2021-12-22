use alloc::collections::BTreeMap;
use libutils::paths::OwnedPath;

use crate::*;

use super::descriptor::*;
use super::signals::SignalType;
use super::signals::SignalDisposition;
use super::stats::*;

use super::PID;

/// Process Data
pub struct ProcessData
{
    pub stack_size: usize, // Stack size in pages
    pub mem: Vec<(*mut u8, usize)>,
    pub next_heap: usize,
    pub descriptors: DescriptorTable,
    pub children: Vec<PID>,
    pub parent_pid: PID,
    pub process_group_id: PID,
    pub cwd: OwnedPath,
    pub cmdline_args: Vec<String>,
    pub mem_stats: MemoryStats,
    pub signal_map: BTreeMap<SignalType, SignalDisposition>,
    pub mmapped_files: BTreeMap<*mut u8, usize>,
    pub return_code_listener: Option<&'static mut u32>
}

impl ProcessData
{
    /// Initialize a fresh process data
    /// Safety: The mem_ptr must be valid or zero
    pub unsafe fn new(stack_size: usize, mem_stats: MemoryStats, pgid: PID) -> Self
    {
        let descriptors: DescriptorTable = BTreeMap::new();
        
        let mut signal_map = BTreeMap::new();

        signal_map.insert(SignalType::SIGTRAP, SignalDisposition::Core);
        signal_map.insert(SignalType::SIGTERM, SignalDisposition::Terminate);
        signal_map.insert(SignalType::SIGSTOP, SignalDisposition::Stop);
        signal_map.insert(SignalType::SIGCONT, SignalDisposition::Continue);
        signal_map.insert(SignalType::SIGKILL, SignalDisposition::Terminate);
        signal_map.insert(SignalType::SIGINT, SignalDisposition::Terminate);

        Self
        {
            stack_size,
            mem: Vec::new(),
            next_heap: 0x4_0000_0000,
            descriptors: descriptors,
            children: Vec::new(),
            parent_pid: 0,
            process_group_id: pgid,
            cwd: OwnedPath::new("/home/root/"),
            cmdline_args: Vec::new(),
            mem_stats,
            signal_map,
            mmapped_files: BTreeMap::new(),
            return_code_listener: None
        }
    }

    /// Fill in the command line arguments
    pub fn fill_command_line_args(&mut self, args: Vec<String>)
    {
        self.cmdline_args = args;
    }

    /// Convert the command line arguments to a single string to display
    pub fn command_line_args_to_string(&self) -> String
    {
        let mut s = String::new();

        for arg in &self.cmdline_args
        {
            if s.len() > 0
            {
                s += "\0";
            }

            s += arg;
        }
        
        s
    }

    /// Remap a file descriptor
    pub fn remap_file_descriptor(&mut self, index: usize, fd: Box<dyn FileDescriptor>)
    {
        self.descriptors.insert(index, alloc::sync::Arc::new(core::cell::RefCell::new(fd)));
    }

    /// Register a child process
    pub fn register_child(&mut self, child_pid: PID)
    {
        self.children.push(child_pid);
    }

    /// Set the parent PID
    pub fn set_parent(&mut self, parent: PID)
    {
        self.parent_pid = parent;
    }
}