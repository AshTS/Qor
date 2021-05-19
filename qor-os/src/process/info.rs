use core::usize;

use crate::*;

use alloc::collections::BTreeMap;

/// Contains the information for a process to interact with the system (cwd,
/// file descriptors, etc)
#[derive(Debug)]
pub struct ProcessInfo
{
    cwd: String,
    file_descriptors: BTreeMap<usize, Option<fs::FilePtr>>
}

impl ProcessInfo
{
    pub fn new() -> Self
    {
        let mut file_descriptors = BTreeMap::new();

        file_descriptors.insert(0, None);
        file_descriptors.insert(1, None);
        file_descriptors.insert(2, None);

        Self
        {
            cwd: String::from("/"),
            file_descriptors
        }
    }

    pub fn open_fd(&mut self, interface: &mut fs::FileSystemInterface, name: &str, _mode: usize) -> usize
    {
        let fd = self.file_descriptors.len();

        kdebugln!(FileSystemSyscall, "Opening File `{}`", name);

        match interface.get_inode_by_name(name)
        {
            Ok(ptr) =>
            {
                self.file_descriptors.insert(fd, Some(ptr));
                fd
            },
            Err(e) =>
            {
                kdebugln!(FileSystemSyscall, "ERROR: {:?}", e);
                0xFFFFFFFFFFFFFFFF
            }
        }
    }

    pub fn close_fd(&mut self, _interface: &mut fs::FileSystemInterface, fd: usize) -> usize
    {
        kdebugln!(FileSystemSyscall, "Closing File Descriptor `{}`", fd);
        
        if self.file_descriptors.insert(fd, None).is_some()
        {
            0
        }
        else
        {
            0xFFFFFFFFFFFFFFFF
        }
    }
}