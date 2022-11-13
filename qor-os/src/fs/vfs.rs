use alloc::sync::Arc;
use alloc::{collections::BTreeMap, string::ToString};
use alloc::boxed::Box;
use atomic::Atomic;
use libutils::paths::{OwnedPath, PathBuffer};
use libutils::sync::{Mutex, InitThreadMarker, MutexGuard};

use crate::{types::DeviceIdentifier, kdebugln, fs::FileSystemError};

use super::{FileSystem, InodeIndex, FilesystemResult};

// Next Device id
static NEXT_DEVICE_ID: Atomic<DeviceIdentifier> = Atomic::new(1);

/// Get the next device identifier
fn next_device_id() -> DeviceIdentifier {
    NEXT_DEVICE_ID.fetch_add(1, core::sync::atomic::Ordering::SeqCst)
}

pub struct FilesystemInterface {
    mounts: BTreeMap<DeviceIdentifier, Option<Box<dyn FileSystem>>>,
    root: Option<DeviceIdentifier>,
    index: BTreeMap<OwnedPath, InodeIndex>,
    indexed: BTreeMap<InodeIndex, OwnedPath>
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
    pub fn mount_fs(&mut self, path: PathBuffer, mut fs: Box<dyn FileSystem>) -> FilesystemResult<DeviceIdentifier> {
        // Get the next device id
        let device_id = next_device_id();
        
        kdebugln!(unsafe Filesystem, "Mounting filesystem with device id {} to {}", device_id, path);

        // Set the mount id on the filesystem
        fs.set_mount_id(device_id, self)?;

        // Get the root inode from the filesystem
        let root_inode = fs.get_root_inode()?;

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

                // TODO: let inode = self.path_to_inode(&path_start)?;
                // TODO: self.mount_fs_at(inode, root_inode, name.to_string());

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
}

impl FileSystem for FilesystemInterface {
    /// Initialize the filesystem on the current disk
    fn init(&mut self) -> FilesystemResult<()> {
        kdebugln!(unsafe Filesystem, "Initialize Virtual Filesystem");

        Ok(())
    }

    /// Sync the filesystem on the current disk
    fn sync(&mut self) -> FilesystemResult<()> {
        kdebugln!(unsafe Filesystem, "Syncing Virtual Filesystem");

        // Sync the filesystem by syncing all of the mounted file systems
        for (_, fs) in &mut self.mounts {
            if let Some(fs) = fs {
                fs.sync()?;
            }
        }

        Ok(())
    }

    /// Set the mount_if of the filesystem
    fn set_mount_id(&mut self, _mount_id: DeviceIdentifier, _interface: &mut FilesystemInterface) -> FilesystemResult<()> {
        panic!("Cannot mount the filesystem interface")
    }

    /// Get the root inode of the filesystem
    fn get_root_inode(&mut self) -> FilesystemResult<super::InodePointer> {
        self.root_fs_error()?.get_root_inode()
    }
}