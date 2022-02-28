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
    BadFilesystemFormat,
    FileNotFound(String),
    OutOfSpace,
    PermissionDenied,
    DirectoryNotEmpty,
    INodeIsDirectory
}

impl FilesystemError
{
    pub fn to_errno(&self) -> usize
    {
        match self
        {
            FilesystemError::MissingRootMount => errno::ENODEV,
            FilesystemError::FilesystemUninitialized => errno::ENODEV,
            FilesystemError::UnableToFindDiskMount(_) => errno::ENODEV,
            FilesystemError::FilesystemNotMounted => errno::ENODEV,
            FilesystemError::INodeIsNotADirectory => errno::ENOTDIR,
            FilesystemError::BadINode => errno::EPERM,
            FilesystemError::BadFilesystemFormat => errno::ENODEV,
            FilesystemError::FileNotFound(_) => errno::ENOENT,
            FilesystemError::OutOfSpace => errno::ENOSPC,
            FilesystemError::PermissionDenied => errno::EPERM,
            FilesystemError::DirectoryNotEmpty => errno::ENOTEMPTY,
            FilesystemError::INodeIsDirectory => errno::EISDIR,
        }
    }
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

/// Stat structure
#[derive(Debug, Clone, Copy)]
pub struct FileStat
{
    pub dev_id: usize,
    pub inode: usize,
    pub mode: u16,
    pub links: u16,
    pub uid: u16,
    pub gid: u16,
    pub special_dev_id: usize,
    pub size: usize,
    pub blk_size: usize,
    pub blocks_alloced: usize,
    pub atime: usize,
    pub mtime: usize,
    pub ctime: usize
}