use crate::fs::{FileMode, Permissions, DirectoryEntryType};
use crate::types::{DeviceIdentifier, TimeRepr};
use super::{FileSystem, FilesystemResult, FilesystemInterface, InodePointer, FileStat, DirectoryEntry};
use alloc::{boxed::Box, collections::BTreeMap};
use alloc::vec;
use alloc::vec::Vec;
use alloc::string::*;

use crate::*;

#[derive(Debug, Clone)]
pub enum RamFSFileData {
    FileData(Vec<u8>),
    DirectoryData(Vec<(alloc::string::String, usize)>)
}

#[derive(Debug, Clone)]
pub struct RamFSInode {
    filestat: FileStat,
    file_data: RamFSFileData
}

#[derive(Debug, Clone)]
pub struct RamFS {
    mount_id: Option<usize>,
    inodes: BTreeMap<usize, RamFSInode>,
    mounted_inodes: Vec<(InodePointer, InodePointer, String)>
}

impl RamFS {
    /// Create a new RamFS Instance
    pub fn new() -> Self {
        Self {
            mount_id: None,
            inodes: BTreeMap::new(),
            mounted_inodes: Vec::new()
        }
    }

    /// Construct an inode within the current device
    pub fn inode(&self, inode: usize) -> FilesystemResult<InodePointer> {
        if let Some(mount_id) = self.mount_id {
            Ok(InodePointer { device_id: mount_id, index: inode })
        }
        else {
            Err(fs::FileSystemError::UnmountedDevice)
        }
    }
}

#[async_trait::async_trait]
impl FileSystem for RamFS {
    /// Initialize the filesystem on the current disk
    async fn init(&mut self) -> FilesystemResult<()> {
        kdebugln!(unsafe "Initializing RamFS");

        Ok(())
    }

    /// Sync the filesystem on the current disk
    async fn sync(&mut self) -> FilesystemResult<()> {
        kdebugln!(unsafe "Syncing RamFS");

        Ok(())
    }

    /// Set the mount_if of the filesystem
    async fn set_mount_id(&mut self, mount_id: DeviceIdentifier, _: &mut FilesystemInterface) -> FilesystemResult<()> {
        kdebugln!(unsafe "Setting mount_id to {}", mount_id);

        self.mount_id = Some(mount_id);

        let hello = FileStat {
            index: self.inode(2)?,
            mode: FileMode::from_components(DirectoryEntryType::RegularFile, Permissions::read_write_execute(), Permissions::read_write_execute(), Permissions::read_write_execute()),
            links: 1,
            uid: 1000,
            gid: 1000,
            special_dev_id: 0,
            size: 0,
            blk_size: 4096,
            blocks_allocated: 1,
            atime: TimeRepr(0),
            mtime: TimeRepr(0),
            ctime: TimeRepr(0),
        };

        let world = FileStat {
            index: self.inode(3)?,
            mode: FileMode::from_components(DirectoryEntryType::RegularFile, Permissions::read_write_execute(), Permissions::read_write_execute(), Permissions::read_write_execute()),
            links: 1,
            uid: 1000,
            gid: 1000,
            special_dev_id: 0,
            size: 0,
            blk_size: 4096,
            blocks_allocated: 1,
            atime: TimeRepr(0),
            mtime: TimeRepr(0),
            ctime: TimeRepr(0),
        };

        self.inodes.insert(2, RamFSInode { filestat: hello, file_data: RamFSFileData::FileData(vec![b'H', b'e', b'l', b'l', b'o']) });
        self.inodes.insert(3, RamFSInode { filestat: world, file_data: RamFSFileData::FileData(vec![b'W', b'o', b'r', b'l', b'd']) });

        let directory = FileStat {
            index: self.inode(1)?,
            mode: FileMode::from_components(DirectoryEntryType::Directory, Permissions::read_write_execute(), Permissions::read_write_execute(), Permissions::read_write_execute()),
            links: 1,
            uid: 1000,
            gid: 1000,
            special_dev_id: 0,
            size: 0,
            blk_size: 4096,
            blocks_allocated: 1,
            atime: TimeRepr(0),
            mtime: TimeRepr(0),
            ctime: TimeRepr(0),
        };

        self.inodes.insert(1, RamFSInode { filestat: directory, file_data: RamFSFileData::DirectoryData(vec![("hello".to_string(), 2), ("world".to_string(), 3)]) });


        Ok(())
    }

    /// Get the root inode of the filesystem
    async fn get_root_inode(&mut self) -> FilesystemResult<InodePointer> {
        kdebugln!(unsafe "Getting the root inode");

        self.inode(1)
    }


    /// Stat the given inode
    async fn stat_inode(&mut self, inode: InodePointer) -> FilesystemResult<FileStat> {
        if self.mount_id == Some(inode.device_id) {
            if let Some(f) = self.inodes.get(&inode.index) {
                Ok(f.filestat)
            }
            else {
                Err(fs::FileSystemError::BadInode(inode))
            }
        }  
        else {
            Err(fs::FileSystemError::UnmountedDevice)
        }
    }
    
    /// Get the directory entries from the given inode
    async fn dir_entries(&mut self, inode: InodePointer) -> FilesystemResult<alloc::vec::Vec<DirectoryEntry>> {
        if self.mount_id == Some(inode.device_id) {
            if let Some(f) = self.inodes.get(&inode.index) {
                let f = f.clone();
                match &f.file_data {
                    RamFSFileData::FileData(_) => Err(fs::FileSystemError::InodeIsADirectory(inode)),
                    RamFSFileData::DirectoryData(entries) => {
                        let mut result = alloc::vec::Vec::new();

                        for (name, index) in entries {
                            let s = self.stat_inode(self.inode(*index)?).await?;
                            result.push(DirectoryEntry { index: self.inode(*index)?, name: name.clone(), entry_type: s.mode.entry_type() })
                        }

                        Ok(result)
                    }
                }
            }
            else {
                Err(fs::FileSystemError::BadInode(inode))
            }
        }  
        else {
            Err(fs::FileSystemError::UnmountedDevice)
        }
    }

    /// Mount a filesystem at the given inode
    async fn mount_fs_at(&mut self, inode: InodePointer, root: InodePointer, name: alloc::string::String) -> FilesystemResult<()> {
        self.mounted_inodes.push((inode, root, name));

        Ok(())
    }
}