use crate::types::{DeviceIdentifier, UserIdentifier, TimeRepr};

/// Inode Index
pub type InodeIndex = usize;

/// Pointer to a device and an inode
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InodePointer {
    pub device_id: DeviceIdentifier,
    pub index: InodeIndex
}

/// Mode structure
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileMode(u16);

/// Directory Entry File Types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectoryEntryType {
    Unknown,
    RegularFile,
    Directory,
    CharDevice,
    BlockDevice,
    FirstInFirstOut,
    Socket,
    SymbolicLink
}

/// Directory Entry
#[derive(Debug, Clone)]
pub struct DirectoryEntry {
    pub index: InodePointer,
    pub name: alloc::string::String,
    pub entry_type: DirectoryEntryType
}

/// File Stat Data
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FileStat {
    pub index: InodePointer,
    pub mode: FileMode,
    pub links: u16,
    pub uid: UserIdentifier,
    pub gid: UserIdentifier,
    pub special_dev_id: DeviceIdentifier,
    pub size: usize,
    pub blk_size: usize,
    pub blocks_allocated: usize,
    pub atime: TimeRepr,
    pub mtime: TimeRepr,
    pub ctime: TimeRepr
}