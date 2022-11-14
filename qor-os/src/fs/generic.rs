use alloc::boxed::Box;

use crate::types::DeviceIdentifier;

use super::{DirectoryEntry, FileMode, FileStat, FilesystemInterface, InodePointer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileSystemError {
    MissingRootMount,
    UnableToFindDevice(DeviceIdentifier),
    UnmountedDevice,
    BadInode(InodePointer),
    InodeIsADirectory(InodePointer),
    UninitializedFilesystem,
    BadPath,
    BadFilesystemFormat,
}

/// Filesystem Result Type
pub type FilesystemResult<T> = Result<T, FileSystemError>;

/// Generic file system interface
#[async_trait::async_trait]
pub trait FileSystem: Send {
    /// Initialize the filesystem on the current disk
    async fn init(&mut self) -> FilesystemResult<()>;

    /// Sync the filesystem on the current disk
    async fn sync(&mut self) -> FilesystemResult<()>;

    /// Set the mount_if of the filesystem
    async fn set_mount_id(
        &mut self,
        mount_id: DeviceIdentifier,
        interface: &mut FilesystemInterface,
    ) -> FilesystemResult<()>;

    /// Get the root inode of the filesystem
    async fn get_root_inode(&mut self) -> FilesystemResult<InodePointer>;

    /// Stat the given inode
    async fn stat_inode(&mut self, inode: InodePointer) -> FilesystemResult<FileStat>;

    /// Get the directory entries from the given inode
    async fn dir_entries(
        &mut self,
        inode: InodePointer,
    ) -> FilesystemResult<alloc::vec::Vec<DirectoryEntry>>;

    /// Mount a filesystem at the given inode
    async fn mount_fs_at(
        &mut self,
        inode: InodePointer,
        root: InodePointer,
        name: alloc::string::String,
    ) -> FilesystemResult<()>;

    /// Allocate a new file with the given mode
    async fn create_file(
        &mut self,
        inode: InodePointer,
        mode: FileMode,
        name: alloc::string::String,
    ) -> FilesystemResult<InodePointer>;

    /// Allocate a new directory
    async fn create_directory(
        &mut self,
        inode: InodePointer,
        name: alloc::string::String,
    ) -> FilesystemResult<InodePointer>;

    /// Remove an inode
    async fn remove_inode(&mut self, inode: InodePointer) -> FilesystemResult<()>;

    /// Increment the number of hard links to an inode
    async fn increment_links(&mut self, inode: InodePointer) -> FilesystemResult<usize>;

    /// Decrement the number of hard links to an inode
    async fn decrement_links(&mut self, inode: InodePointer) -> FilesystemResult<usize>;

    /// Read the data from an inode
    async fn read_inode(&mut self, inode: InodePointer) -> FilesystemResult<alloc::vec::Vec<u8>>;

    /// Write data to an inode
    async fn write_inode(&mut self, inode: InodePointer, data: &[u8]) -> FilesystemResult<()>;
}
