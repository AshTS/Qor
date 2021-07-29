use crate::*;

use super::fstrait::Filesystem;
use super::structures::*;

use alloc::vec;

use crate::process::descriptor::*;

use libutils::paths::PathBuffer;

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
    fn init(&mut self) -> FilesystemResult<()>
    {
        // Add the null inode
        if self.inodes.len() < 1
        {
            let id = 1;

            self.inodes.push(RamDiskInode::Null);

            let directory = RamDiskInode::Directory(String::from(""), Vec::new());

            self.inodes.push(directory);

            let directory = RamDiskInode::Directory(String::from("dir"), Vec::new());

            self.inodes.push(directory);

            let file = RamDiskInode::File(String::from("file"), vec!['H' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8]);
        
            self.inodes.push(file);

            let file = RamDiskInode::File(String::from("file2"), vec!['H' as u8, 'e' as u8, 'y' as u8, '!' as u8]);
        
            self.inodes.push(file);

            self.inodes[1] = RamDiskInode::Directory(String::from(""), vec![
                (String::from("file"), FilesystemIndex{inode: 3, mount_id: id}),
                (String::from("."), FilesystemIndex{inode: 1, mount_id: id}),
                (String::from(".."), FilesystemIndex{inode: 1, mount_id: id}),
                (String::from("dir"), FilesystemIndex{inode: 2, mount_id: id})
                ]);

            self.inodes[2] = RamDiskInode::Directory(String::from("dir"), vec![
                (String::from("."), FilesystemIndex{inode: 2, mount_id: id}),
                (String::from(".."), FilesystemIndex{inode: 1, mount_id: id}),
                (String::from("file2"), FilesystemIndex{inode: 4, mount_id: id})
                ]);
        }

        Ok(())
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
    fn path_to_inode(&mut self, path: PathBuffer) -> FilesystemResult<FilesystemIndex>
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
    fn inode_to_path(&mut self, inode: FilesystemIndex) -> FilesystemResult<PathBuffer>
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
                    Err(FilesystemError::INodeIsNotADirectory)
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

    /// Create a file in the directory at the given inode
    fn create_file(&mut self, inode: FilesystemIndex, name: String) -> FilesystemResult<FilesystemIndex>
    {
        if Some(inode.mount_id) == self.mount_id
        {
            let new_file = RamDiskInode::File(name.clone(), Vec::new());
            let next_id = self.inodes.len();

            self.inodes.push(new_file);

                
            if let Some(inode) = self.inodes.iter_mut().nth(inode.inode)
            {
                if let RamDiskInode::Directory(_, children) = inode
                {
                    let index = FilesystemIndex{ mount_id: self.mount_id.unwrap(), inode: next_id };

                    children.push((name, index));

                    Ok(index)
                }
                else
                {
                    Err(FilesystemError::INodeIsNotADirectory)
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
                (*vfs).create_file(inode, name)
            }
            else
            {
                Err(FilesystemError::FilesystemNotMounted)
            }
        }
    }

    /// Create a directory in the directory at the given inode
    fn create_directory(&mut self, inode: FilesystemIndex, name: String) -> FilesystemResult<FilesystemIndex>
    {
        if Some(inode.mount_id) == self.mount_id
        {
            let next_id = self.inodes.len();
            let index = FilesystemIndex{ mount_id: self.mount_id.unwrap(), inode: next_id };

            let new_dir = RamDiskInode::Directory(name.clone(), vec![
                (String::from("."), index),
                (String::from(".."), inode)
            ]);

            self.inodes.push(new_dir);

                
            if let Some(inode) = self.inodes.iter_mut().nth(inode.inode)
            {
                if let RamDiskInode::Directory(_, children) = inode
                {
                    children.push((name, index));

                    Ok(index)
                }
                else
                {
                    Err(FilesystemError::INodeIsNotADirectory)
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
                (*vfs).create_file(inode, name)
            }
            else
            {
                Err(FilesystemError::FilesystemNotMounted)
            }
        }
    }

    /// Remove an inode at the given index from the given directory
    fn remove_inode(&mut self, inode: FilesystemIndex, directory: FilesystemIndex) -> FilesystemResult<()>
    {
        if Some(inode.mount_id) == self.mount_id
        {
            if let Some(val) = self.inodes.get_mut(inode.inode)
            {
                *val = RamDiskInode::Null;

                // TODO: Remove the reference from the directory

                Ok(())
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
                (*vfs).remove_inode(inode, directory)
            }
            else
            {
                Err(FilesystemError::FilesystemNotMounted)
            }
        }
    }

    /// Read the data stored in an inode
    fn read_inode(&mut self, inode: FilesystemIndex) -> FilesystemResult<Vec<u8>>
    {
        if Some(inode.mount_id) == self.mount_id
        {
            if let RamDiskInode::File(_, data) = &self.inodes[inode.inode]
            {
                Ok(data.clone())
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
                vfs.read_inode(inode)
            }
            else
            {
                Err(FilesystemError::FilesystemNotMounted)
            }
        }
    }

    /// Write data to an inode
    fn write_inode(&mut self, _inode: FilesystemIndex, _data: &[u8]) -> FilesystemResult<()>
    {
        todo!()
    }

    /// Mount a filesystem at the given inode
    fn mount_fs_at(&mut self, _inode: FilesystemIndex, _root: FilesystemIndex, _name: String) -> FilesystemResult<()>
    {
        todo!()
    }

    /// Open a filedescriptor for the given inode
    fn open_fd(&mut self, inode: FilesystemIndex, mode: usize) -> FilesystemResult<Box<dyn crate::process::descriptor::FileDescriptor>>
    {
        if let Some(vfs) = &mut self.vfs
        {
            if Some(inode.mount_id) == self.mount_id
            {
                if inode.inode < self.inodes.len()
                {
                    Ok(Box::new(InodeFileDescriptor::new(vfs, inode, mode).unwrap()))
                }
                else
                {
                    Err(FilesystemError::BadINode)
                }
            }
            else
            {
                vfs.open_fd(inode, mode)   
            }
        }
        else
        {
            Err(FilesystemError::FilesystemNotMounted)
        }
    }
}