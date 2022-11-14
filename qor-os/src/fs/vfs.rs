use alloc::{collections::BTreeMap, string::ToString};
use alloc::boxed::Box;
use atomic::Atomic;
use libutils::paths::{OwnedPath, PathBuffer};
use alloc::format;
use crate::*;

use crate::{types::DeviceIdentifier, kdebugln, fs::FileSystemError};

use super::{FileSystem, FilesystemResult, InodePointer, FileStat, DirectoryEntry, FileMode};

// Next Device id
static NEXT_DEVICE_ID: Atomic<DeviceIdentifier> = Atomic::new(1);

/// Get the next device identifier
fn next_device_id() -> DeviceIdentifier {
    NEXT_DEVICE_ID.fetch_add(1, core::sync::atomic::Ordering::SeqCst)
}

pub struct FilesystemInterface {
    mounts: BTreeMap<DeviceIdentifier, Option<Box<dyn FileSystem>>>,
    root: Option<DeviceIdentifier>,
    index: BTreeMap<OwnedPath, InodePointer>,
    indexed: BTreeMap<InodePointer, OwnedPath>
}

impl FilesystemInterface {
    /// Construct a new filesystem interface
    pub fn new() -> Self {
        Self {
            mounts: BTreeMap::new(),
            root: None,
            index: BTreeMap::new(),
            indexed: BTreeMap::new()
        }
    }

    /// Mount a filesystem
    pub async fn mount_fs(&mut self, path: OwnedPath, mut fs: Box<dyn FileSystem>) -> FilesystemResult<DeviceIdentifier> {
        // Get the next device id
        let device_id = next_device_id();
        
        kdebugln!(unsafe Filesystem, "Mounting filesystem with device id {} to {}", device_id, path);

        // Set the mount id on the filesystem
        fs.set_mount_id(device_id, self).await?;

        // Get the root inode from the filesystem
        let root_inode = fs.get_root_inode().await?;

        // Insert the device
        self.mounts.insert(device_id, Some(fs));

        // Add the mapping into the mount paths
        if path.as_str() == "/" {
            self.root = Some(device_id);

            Ok(device_id)
        }
        else {
            if self.root.is_some() {
                let (path_start, name) = path.split_last();

                let inode = self.path_to_inode(&path_start)?;
                self.mount_fs_at(inode, root_inode, name.to_string()).await?;

                Ok(device_id)
            }
            else {
                Err(FileSystemError::MissingRootMount)
            }
        }
    }

    /// Get the filesystem mounted for the given device id
    pub fn fs_from_device(&mut self, id: DeviceIdentifier) -> Option<&mut Box<dyn FileSystem>> {
        self.mounts.get_mut(&id).map(|v| v.as_mut())?
    }

    /// Get the filesystem mounted for the given device id or an error
    pub fn fs_from_device_error(&mut self, id: DeviceIdentifier) -> FilesystemResult<&mut Box<dyn FileSystem>> {
        self.fs_from_device(id).ok_or(FileSystemError::UnableToFindDevice(id))
    }

    /// Get the root filesystem
    pub fn root_fs(&mut self) -> Option<&mut Box<dyn FileSystem>> {
        self.root.map(|id| self.fs_from_device(id))?
    }

    /// Get the root filesystem or an error
    pub fn root_fs_error(&mut self) -> FilesystemResult<&mut Box<dyn FileSystem>> {
        self.root.ok_or(FileSystemError::MissingRootMount).map(|id| self.fs_from_device_error(id))?
    }

    /// Index the filesystem starting at the given inode, giving it the desired path
    #[async_recursion::async_recursion]
    pub async fn index_from(&mut self, path: OwnedPath, inode: InodePointer) -> FilesystemResult<()> {
        // If we have already indexed this path, skip
        if self.indexed.contains_key(&inode) {
            return Ok(());
        }

        // Put the current path into the index
        if !path.as_str().is_empty() {
            self.index.insert(path.clone(), inode);
        }
        // Do the reverse as well
        self.indexed.insert(inode, path.clone());

        // Iterate down through the directory entries
        match self.dir_entries(inode).await {
            Ok(entries) => {
                self.index.insert(OwnedPath::new(path.as_str().to_string() + "/"), inode);

                for entry in entries {
                    self.index_from(OwnedPath::new(format!("{}/{}", path, entry.name)), entry.index).await?;
                }

                Ok(())
            },
            Err(FileSystemError::InodeIsADirectory(_)) => Ok(()),
            Err(e) => Err(e)
        }
    }

    /// Index the entire filesystem
    pub async fn index(&mut self) -> FilesystemResult<()> {
        // Clear out the current index
        self.index.clear();
        self.indexed.clear();

        // Index starting from the root
        let root = self.get_root_inode().await?;
        self.index_from("".into(), root).await?;

        kprintln!(unsafe "{:#?}", self.index);
        Ok(())
    }

    /// Invalidate part of the index
    pub fn invalidate_index(&mut self, path: PathBuffer) -> FilesystemResult<()> {
        let mut to_remove = alloc::vec::Vec::new();

        for p in self.index.keys() {
            if p.as_str().starts_with(path.as_str()) {
                to_remove.push(p.clone());
            }
        }

        for path in to_remove {
            self.index.remove(&path);
        }

        Ok(())
    }

    /// Path to inode
    pub fn path_to_inode(&mut self, path: PathBuffer) -> FilesystemResult<InodePointer> {
        self.index.get(&path).ok_or(FileSystemError::BadPath).copied()
    }

    /// Attempt to get a path from an inode
    pub fn inode_to_path(&mut self, inode: InodePointer) -> FilesystemResult<PathBuffer> {
        self.indexed.get(&inode).ok_or(FileSystemError::BadInode(inode))
    }
}

#[async_trait::async_trait]
impl FileSystem for FilesystemInterface {
    /// Initialize the filesystem on the current disk
    async fn init(&mut self) -> FilesystemResult<()> {
        kdebugln!(unsafe Filesystem, "Initialize Virtual Filesystem");

        Ok(())
    }

    /// Sync the filesystem on the current disk
    async fn sync(&mut self) -> FilesystemResult<()> {
        kdebugln!(unsafe Filesystem, "Syncing Virtual Filesystem");

        // Sync the filesystem by syncing all of the mounted file systems
        for (_, fs) in &mut self.mounts {
            if let Some(fs) = fs {
                fs.sync().await?;
            }
        }

        Ok(())
    }

    /// Set the mount_if of the filesystem
    async fn set_mount_id(&mut self, _mount_id: DeviceIdentifier, _interface: &mut FilesystemInterface) -> FilesystemResult<()> {
        panic!("Cannot mount the filesystem interface")
    }

    /// Get the root inode of the filesystem
    async fn get_root_inode(&mut self) -> FilesystemResult<super::InodePointer> {
        self.root_fs_error()?.get_root_inode().await
    }

    /// Stat the given inode
    async fn stat_inode(&mut self, inode: InodePointer) -> FilesystemResult<FileStat> {
        self.fs_from_device_error(inode.device_id)?.stat_inode(inode).await
    }

    /// Get the directory entries from the given inode
    async fn dir_entries(&mut self, inode: InodePointer) -> FilesystemResult<alloc::vec::Vec<DirectoryEntry>> {
        self.fs_from_device_error(inode.device_id)?.dir_entries(inode).await
    }

    /// Mount a filesystem at the given inode
    async fn mount_fs_at(&mut self, inode: InodePointer, root: InodePointer, name: alloc::string::String) -> FilesystemResult<()> {
        self.fs_from_device_error(inode.device_id)?.mount_fs_at(inode, root, name).await
    }

    /// Allocate a new file with the given mode
    async fn create_file(&mut self, inode: InodePointer, mode: FileMode, name: alloc::string::String) -> FilesystemResult<InodePointer> {
        self.fs_from_device_error(inode.device_id)?.create_file(inode, mode, name).await
    }

    /// Allocate a new directory
    async fn create_directory(&mut self, inode: InodePointer, name: alloc::string::String) -> FilesystemResult<InodePointer> {
        self.fs_from_device_error(inode.device_id)?.create_directory(inode, name).await
    }

    /// Remove an inode
    async fn remove_inode(&mut self, inode: InodePointer) -> FilesystemResult<()> {
        self.fs_from_device_error(inode.device_id)?.remove_inode(inode).await
    }

    /// Increment the number of hard links to an inode
    async fn increment_links(&mut self, inode: InodePointer) -> FilesystemResult<usize> {
        self.fs_from_device_error(inode.device_id)?.increment_links(inode).await
    }

    /// Decrement the number of hard links to an inode
    async fn decrement_links(&mut self, inode: InodePointer) -> FilesystemResult<usize> {
        self.fs_from_device_error(inode.device_id)?.decrement_links(inode).await
    }

    /// Read the data from an inode
    async fn read_inode(&mut self, inode: InodePointer) -> FilesystemResult<alloc::vec::Vec<u8>> {
        self.fs_from_device_error(inode.device_id)?.read_inode(inode).await
    }

    /// Write data to an inode
    async fn write_inode(&mut self, inode: InodePointer, data: &[u8]) -> FilesystemResult<()> {
        self.fs_from_device_error(inode.device_id)?.write_inode(inode, data).await
    }
}