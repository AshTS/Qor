use crate::*;

use super::super::fstrait::*;
use super::super::structures::*;

use libutils::paths::PathBuffer;

use crate::process::descriptor::*;

use super::devices::*;

use super::super::ioctl::*;

/// Filesystem which gives access to various devices
pub struct DevFilesystem
{
    mount_id: Option<usize>,
    vfs: Option<&'static mut crate::fs::vfs::FilesystemInterface>,
    devices: Vec<DeviceFile>,
    directories: Vec<DeviceDirectories>
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
            devices: Vec::new(),
            directories: Vec::new(),
        }
    }

    /// Get directory entries for a given directory
    pub fn get_directory_entries_for(&self, mount_id: usize, directory: DeviceDirectories) -> Vec<DirectoryEntry>
    {
        let mut result = Vec::new();

        // Construct the loop back and parent entries
        let loopback = DirectoryEntry{
            index: FilesystemIndex { mount_id, inode: 1},
            name: String::from("."),
            entry_type: DirectoryEntryType::Directory,
        };

        let parent = DirectoryEntry{
            index: FilesystemIndex { mount_id, inode: 1},
            name: String::from(".."),
            entry_type: DirectoryEntryType::Directory,
        };

        result.push(loopback);
        result.push(parent);

        for (i, dev) in self.devices.iter().enumerate()
        {
            if dev.directory != directory { continue; }

            let dir_ent = DirectoryEntry
                {
                    index: FilesystemIndex { mount_id, inode: i + 2 + self.directories.len()},
                    name: String::from(dev.name),
                    entry_type: DirectoryEntryType::CharDevice,
                };

            result.push(dir_ent);
        }

        // Specific cases for getting the entries in a directory
        if directory == DeviceDirectories::PseudoTerminalSecondaries
        {
            for index in super::tty::get_open_pseudo_terminal_indexes()
            {
                let dir_ent = DirectoryEntry
                {
                    index: FilesystemIndex { mount_id, inode: PSUEDO_TERMINAL_FLAG | index},
                    name: format!("{}", index),
                    entry_type: DirectoryEntryType::CharDevice,
                };

                result.push(dir_ent);
            }
        }

        result
    }
}

impl Filesystem for DevFilesystem
{
    fn init(&mut self) -> FilesystemResult<()>
    {
        // Set up the devices available on the system
        self.devices = get_device_files();

        // Set up the device directories
        self.directories = get_device_directories();

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
                let mut result = self.get_directory_entries_for(inode.mount_id, DeviceDirectories::Root);

                for (i, dir) in self.directories.iter().enumerate()
                {
                    let dir_ent = DirectoryEntry
                    {
                        index: FilesystemIndex { mount_id: inode.mount_id, inode: i + 2},
                        name: format!("{}", dir),
                        entry_type: DirectoryEntryType::CharDevice,
                    };

                result.push(dir_ent);
                }

                Ok(result)
            }
            else if inode.inode < 2 + self.directories.len()
            {
                Ok(self.get_directory_entries_for(inode.mount_id, self.directories[inode.inode - 2]))
            }
            else if inode.inode < 2 + self.directories.len() + self.devices.len()
            {
                Err(FilesystemError::INodeIsNotADirectory)
            }
            else if inode.inode & PSUEDO_TERMINAL_FLAG > 0
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
            if inode.inode < 2 + self.directories.len() + self.devices.len() ||
                inode.inode & PSUEDO_TERMINAL_FLAG > 0
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
                        if default > 1 && default < 2 + self.directories.len()
                        {
                            Ok(Box::new(InodeFileDescriptor::new(vfs, inode, mode).unwrap()))
                        }
                        else if default >= 2 + self.directories.len() && default < 2 + self.directories.len() + self.devices.len()
                        {
                            Ok(self.devices[default - 2 - self.directories.len()].make_descriptor(inode))
                        }
                        else if default & PSUEDO_TERMINAL_FLAG > 0
                        {
                            super::tty::get_pseudo_terminal_secondary_file_descriptor(default & ((1 << 16) - 1), inode)
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

    /// Execute an ioctl command on an inode
    fn exec_ioctl(&mut self, inode: FilesystemIndex, cmd: IOControlCommand) -> FilesystemResult<usize>
    {
        if let Some(vfs) = &mut self.vfs
        {
            if Some(inode.mount_id) == self.mount_id
            {
                if inode.inode > 1 + self.directories.len() && inode.inode < 2 + self.directories.len() + self.devices.len()
                {
                    Ok(self.devices[inode.inode - 2 - self.directories.len()].exec_ioctl(cmd))
                }
                else
                {
                    Err(FilesystemError::BadINode)
                }
            }
            else
            {
                vfs.exec_ioctl(inode, cmd)   
            }
        }
        else
        {
            Err(FilesystemError::FilesystemNotMounted)
        }
    }
}