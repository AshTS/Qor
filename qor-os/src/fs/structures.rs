//! Filesystem Structures

use crate::*;

/// Filesystem Error
#[derive(Debug, Clone)]
pub enum FilesystemError
{
    MissingRootMount,
    FilesystemUninitialized,
    UnableToFindDiskMount(usize),
    FilesystemNotMounted,
    INodeIsNotADirectory,
    BadINode,
    FileNotFound(String)
}

/// Generic Filesystem Result Type
pub type FilesystemResult<T> = Result<T, FilesystemError>;


/// Filesystem Index
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FilesystemIndex
{
    pub mount_id: usize,
    pub inode: usize
}

/// Directory Entry Types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectoryEntryType
{
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
pub struct DirectoryEntry
{
    pub index: FilesystemIndex,
    pub name: String,
    pub entry_type: DirectoryEntryType
}