//! The dev filesystem is to be mounted at /dev/ and gives access to various
//! devices

use crate::*;

use super::fstrait::*;
use super::structures::*;

use libutils::paths::PathBuffer;

use crate::process::descriptor::*;

/// Filesystem which gives access to various devices
pub struct DevFilesystem
{
    mount_id: Option<usize>,
    vfs: Option<&'static mut crate::fs::vfs::FilesystemInterface>
}

impl DevFilesystem
{
    /// Create a new dev filesystem
    pub fn new() -> Self
    {
        Self
        {
            mount_id: None,
            vfs: None
        }
    }
}

impl Filesystem for DevFilesystem
{
    fn init(&mut self) -> super::structures::FilesystemResult<()>
    {
        // No initialization required
        Ok(())
    }

    fn sync(&mut self) -> super::structures::FilesystemResult<()>
    {
        // Nothing to sync
        Ok(())
    }

    fn set_mount_id(&mut self, mount_id: usize, vfs: &'static mut crate::fs::vfs::FilesystemInterface)
    {
        self.mount_id = Some(mount_id);
        self.vfs = Some(vfs);
    }

    fn get_root_index(&mut self) -> super::structures::FilesystemResult<super::structures::FilesystemIndex>
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

    fn get_dir_entries(&mut self, inode: super::structures::FilesystemIndex) -> super::structures::FilesystemResult<alloc::vec::Vec<super::structures::DirectoryEntry>>
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

                // Construct the entry for the display
                let display = DirectoryEntry{
                    index: FilesystemIndex { mount_id: inode.mount_id, inode: 2},
                    name: String::from("disp"),
                    entry_type: DirectoryEntryType::CharDevice,
                };

                result.push(display);

                Ok(result)
            }
            else if inode.inode == 2
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

    fn create_file(&mut self, _inode: super::structures::FilesystemIndex, _name: alloc::string::String) -> super::structures::FilesystemResult<super::structures::FilesystemIndex>
    {
        todo!()
    }

    fn create_directory(&mut self, _inode: super::structures::FilesystemIndex, _name: alloc::string::String) -> super::structures::FilesystemResult<super::structures::FilesystemIndex>
    {
        todo!()
    }

    fn remove_inode(&mut self, _inode: super::structures::FilesystemIndex, _directory: super::structures::FilesystemIndex) -> super::structures::FilesystemResult<()>
    {
        todo!()
    }

    fn read_inode(&mut self, inode: super::structures::FilesystemIndex) -> super::structures::FilesystemResult<alloc::vec::Vec<u8>>
    {
        if Some(inode.mount_id) == self.mount_id
        {
            if inode.inode < 3
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

    fn write_inode(&mut self, inode: super::structures::FilesystemIndex, data: &[u8]) -> super::structures::FilesystemResult<()>
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

    fn mount_fs_at(&mut self, _inode: super::structures::FilesystemIndex, _root: super::structures::FilesystemIndex, _name: alloc::string::String) -> super::structures::FilesystemResult<()>
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
                    2 => Ok(Box::new(ByteInterfaceDescriptor::new(drivers::gpu::get_global_graphics_driver()))),
                    _ => Err(FilesystemError::BadINode)
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