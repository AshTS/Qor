use crate::types::DeviceIdentifier;

use super::{InodePointer, FilesystemInterface};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileSystemError {
    MissingRootMount,
    UnableToFindDevice(DeviceIdentifier)
}

/// Filesystem Result Type
pub type FilesystemResult<T> = Result<T, FileSystemError>;

/// Generic file system interface
pub trait FileSystem {
    /// Initialize the filesystem on the current disk
    fn init(&mut self) -> FilesystemResult<()>;

    /// Sync the filesystem on the current disk
    fn sync(&mut self) -> FilesystemResult<()>;

    /// Set the mount_if of the filesystem
    fn set_mount_id(&mut self, mount_id: DeviceIdentifier, interface: &mut FilesystemInterface) -> FilesystemResult<()>;

    /// Get the root inode of the filesystem
    fn get_root_inode(&mut self) -> FilesystemResult<InodePointer>;
}