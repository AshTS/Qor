use crate::*;

use super::fstrait::Filesystem;
use super::structures::*;

use alloc::vec;

/// Ram Disk INode
pub enum RamDiskInode
{
    Directory(String, Vec<(String, FilesystemIndex)>),
    File(String, Vec<u8>),
    Null
}

/// Ram Disk Filesystem
pub struct RamDiskFilesystem
{
    inodes: Vec<RamDiskInode>,
    mount_id: Option<usize>,
    vfs: Option<&'static mut crate::fs::vfs::FilesystemInterface>
}

impl RamDiskFilesystem
{
    /// Create a new RamDisk
    pub fn new() -> Self
    {
        Self
        {
            inodes: Vec::new(),
            mount_id: None,
            vfs: None
        }
    }
}

impl Filesystem for RamDiskFilesystem
{
    /// Initialize the filesystem on the current disk
    fn init(&mut self)
    {
        // Add the null inode
        if self.inodes.len() < 1
        {
            self.inodes.push(RamDiskInode::Null);

            let directory = RamDiskInode::Directory(String::from("/"), Vec::new());

            self.inodes.push(directory);

            let file = RamDiskInode::File(String::from("file"), vec!['H' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8]);
        
            self.inodes.push(file);

            self.inodes[1] = RamDiskInode::Directory(String::from("/"), vec![
                (String::from("file"), FilesystemIndex{inode: 2, mount_id: 0}),
                (String::from("."), FilesystemIndex{inode: 1, mount_id: 0}),
                (String::from(".."), FilesystemIndex{inode: 1, mount_id: 0})
                ]);
        }
    }

    /// Sync the filesystem with the current disk
    fn sync(&mut self) -> FilesystemResult<()>
    {
        // No need to sync this filesystem as it is stored entirely in ram
        Ok(())
    }

    /// Set the mount_id of the filesystem
    fn set_mount_id(&mut self, mount_id: usize, vfs: &'static mut crate::fs::vfs::FilesystemInterface)
    {
        self.mount_id = Some(mount_id);
        self.vfs = Some(vfs);
    }

    /// Get the index of the root directory of the filesystem
    fn get_root_index(&mut self) -> FilesystemResult<FilesystemIndex>
    {
        if let Some(id) = self.mount_id
        {
            Ok(FilesystemIndex
            {
                mount_id: id,
                inode: 1
            })
        }
        else
        {
            Err(FilesystemError::FilesystemUninitialized)
        }
    }

    /// Convert a path to an inode
    fn path_to_inode(&mut self, path: &str) -> FilesystemResult<FilesystemIndex>
    {
        if let Some(vfs) = &mut self.vfs
        {
            vfs.path_to_inode(path)
        }
        else
        {
            Err(FilesystemError::FilesystemNotMounted)
        }
    }

    /// Convert an inode to a path
    fn inode_to_path(&mut self, inode: FilesystemIndex) -> FilesystemResult<&str>
    {
        if let Some(vfs) = &mut self.vfs
        {
            vfs.inode_to_path(inode)
        }
        else
        {
            Err(FilesystemError::FilesystemNotMounted)
        }
    }

    /// Get the directory entries for the given inode
    fn get_dir_entries(&mut self, inode: FilesystemIndex) -> FilesystemResult<Vec<DirectoryEntry>>
    {
        if Some(inode.mount_id) == self.mount_id
        {
            if let Some(inode) = self.inodes.iter().nth(inode.inode)
            {
                if let RamDiskInode::Directory(_, children) = inode
                {
                    let mut result = Vec::new();

                    for child in children
                    {
                        let entry = DirectoryEntry{
                            index: child.1,
                            name: child.0.clone(),
                            entry_type: DirectoryEntryType::Unknown,
                        };

                        result.push(entry);
                    }

                    Ok(result)
                }
                else
                {
                    Err(FilesystemError::INodeIsADirectory)
                }
            }
            else
            {
                Err(FilesystemError::BadINode)
            }
        }
        else
        {
            if let Some(vfs) = &mut self.vfs
            {
                (*vfs).get_dir_entries(inode)
            }
            else
            {
                Err(FilesystemError::FilesystemNotMounted)
            }
        }
    }
}