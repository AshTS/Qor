use alloc::collections::BTreeMap;
use libutils::paths::OwnedPath;

use crate::*;

use super::descriptor::*;

/// Process Data
pub struct ProcessData
{
    pub stack_size: usize, // Stack size in pages
    pub mem: Vec<(*mut u8, usize)>,
    pub next_heap: usize,
    pub descriptors: DescriptorTable,
    pub children: Vec<u16>,
    pub parent_pid: u16,
    pub cwd: OwnedPath,
}

impl ProcessData
{
    /// Initialize a fresh process data
    /// Safety: The mem_ptr must be valid or zero
    pub unsafe fn new(stack_size: usize) -> Self
    {
        let descriptors: BTreeMap<usize, Box<dyn FileDescriptor>> = BTreeMap::new();

        Self
        {
            stack_size,
            mem: Vec::new(),
            next_heap: 0x4_0000_0000,
            descriptors: alloc::sync::Arc::new(core::cell::RefCell::new(descriptors)),
            children: Vec::new(),
            parent_pid: 0,
            cwd: OwnedPath::new("/root/")
        }
    }

    /// Remap a file descriptor
    pub fn remap_file_descriptor(&mut self, index: usize, fd: Box<dyn FileDescriptor>)
    {
        self.descriptors.borrow_mut().insert(index, fd);
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