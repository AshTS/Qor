use crate::*;

use super::super::fstrait::*;
use super::super::structures::*;

use libutils::paths::PathBuffer;

use crate::process::descriptor::*;

use super::devices::*;

/// Filesystem which gives access to various devices
pub struct DevFilesystem
{
    mount_id: Option<usize>,
    vfs: Option<&'static mut crate::fs::vfs::FilesystemInterface>,
    devices: Vec<DeviceFile>
}

impl DevFilesystem
{
    /// Create a new dev filesystem
    pub fn new() -> Self
    {
        Self
        {
            mount_id: None,
            vfs: None,
            devices: Vec::new()
        }
    }
}

impl Filesystem for DevFilesystem
{
    fn init(&mut self) -> FilesystemResult<()>
    {
        // Set up the devices available on the system
        self.devices = get_device_files();

        Ok(())
    }

    fn sync(&mut self) -> FilesystemResult<()>
    {
        // Nothing to sync
        Ok(())
    }

    fn set_mount_id(&mut self, mount_id: usize, vfs: &'static mut crate::fs::vfs::FilesystemInterface)
    {
        self.mount_id = Some(mount_id);
        self.vfs = Some(vfs);
    }

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

    fn get_dir_entries(&mut self, inode: FilesystemIndex) -> FilesystemResult<alloc::vec::Vec<DirectoryEntry>>
    {
        if Some(inode.mount_id) == self.mount_id
        {
            if inode.inode == 1
            {
                let mut result = Vec::new();

                // Construct the loop back and parent entries
                let loopback = DirectoryEntry{
                    index: FilesystemIndex { mount_id: inode.mount_id, inode: 1},
                    name: String::from("."),
                    entry_type: DirectoryEntryType::Directory,
                };

                let parent = DirectoryEntry{
                    index: FilesystemIndex { mount_id: inode.mount_id, inode: 1},
                    name: String::from(".."),
                    entry_type: DirectoryEntryType::Directory,
                };

                result.push(loopback);
                result.push(parent);

                for (i, (dev_name, _)) in self.devices.iter().enumerate()
                {
                    let dir_ent = DirectoryEntry
                        {
                            index: FilesystemIndex { mount_id: inode.mount_id, inode: i + 2},
                            name: String::from(*dev_name),
                            entry_type: DirectoryEntryType::CharDevice,
                        };

                    result.push(dir_ent);
                }

                Ok(result)
            }
            else if inode.inode < 4
            {
                Err(FilesystemError::INodeIsNotADirectory)
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

    fn create_file(&mut self, _inode: FilesystemIndex, _name: alloc::string::String) -> FilesystemResult<FilesystemIndex>
    {
        todo!()
    }

    fn create_directory(&mut self, _inode: FilesystemIndex, _name: alloc::string::String) -> FilesystemResult<FilesystemIndex>
    {
        todo!()
    }

    fn remove_inode(&mut self, _inode: FilesystemIndex, _directory: FilesystemIndex) -> FilesystemResult<()>
    {
        todo!()
    }

    fn read_inode(&mut self, inode: FilesystemIndex) -> FilesystemResult<alloc::vec::Vec<u8>>
    {
        if Some(inode.mount_id) == self.mount_id
        {
            if inode.inode < 4
            {
                Ok(Vec::new())
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

    fn write_inode(&mut self, inode: FilesystemIndex, data: &[u8]) -> FilesystemResult<()>
    {
        if Some(inode.mount_id) == self.mount_id
        {
            // If an inode is written to, just dump the data, it doesn't need to
            // be stored

            Ok(())
        }
        else
        {
            if let Some(vfs) = &mut self.vfs
            {
                vfs.write_inode(inode, data)
            }
            else
            {
                Err(FilesystemError::FilesystemNotMounted)
            }
        }
    }

    fn mount_fs_at(&mut self, _inode: FilesystemIndex, _root: FilesystemIndex, _name: alloc::string::String) -> FilesystemResult<()>
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
                match inode.inode
                {
                    1 => Ok(Box::new(InodeFileDescriptor::new(vfs, inode, mode).unwrap())),
                    default =>
                    {
                        if default > 1 && default < 2 + self.devices.len()
                        {
                            Ok(self.devices[default - 2].1())
                        }
                        else
                        {
                            Err(FilesystemError::BadINode)
                        }
                    }
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