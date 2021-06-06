use alloc::collections::BTreeMap;

use crate::*;

/// Process Data
pub struct ProcessData
{
    pub stack_size: usize, // Stack size in pages
    pub mem: Vec<(*mut u8, usize)>,
    pub next_heap: usize,
    pub descriptors: BTreeMap<usize, Box<dyn super::descriptor::FileDescriptor>>,
    pub children: Vec<u16>,
    pub parent_pid: u16,
    pub cwd: String,
}

impl ProcessData
{
    /// Initialize a fresh process data
    /// Safety: The mem_ptr must be valid or zero
    pub unsafe fn new(stack_size: usize) -> Self
    {
        let mut descriptors: BTreeMap<usize, Box<dyn super::descriptor::FileDescriptor>> = BTreeMap::new();

        descriptors.insert(0, Box::new(super::descriptor::NullDescriptor{}));
        descriptors.insert(1, Box::new(super::descriptor::NullDescriptor{}));
        descriptors.insert(2, Box::new(super::descriptor::NullDescriptor{}));

        Self
        {
            stack_size,
            mem: Vec::new(),
            next_heap: 0x4_0000_0000,
            descriptors,
            children: Vec::new(),
            parent_pid: 0,
            cwd: String::from("/bin/")
        }
    }

    /// Connect the process to stdin, stderr, and stdout
    pub fn connect_to_term(&mut self)
    {
        self.descriptors.insert(0, Box::new(super::descriptor::UARTIn{}));
        self.descriptors.insert(1, Box::new(super::descriptor::UARTOut{}));
        self.descriptors.insert(2, Box::new(super::descriptor::UARTError{}));
    }

    /// Register a child process
    pub fn register_child(&mut self, child_pid: u16)
    {
        self.children.push(child_pid);
    }

    /// Set the parent PID
    pub fn set_parent(&mut self, parent: u16)
    {
        self.parent_pid = parent;
    }
}