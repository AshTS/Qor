use crate::*;

use super::super::fstrait::*;
use super::super::structures::*;

use libutils::paths::PathBuffer;

use crate::process::descriptor::*;

const PROC_INODE_FLAG_PID: usize = 0x10000;
const PROC_INODE_FLAG_PID_CMDLINE: usize = 0x20000;
const PROC_INODE_FLAG_PID_STATM: usize = 0x40000;

/// /proc Filesystem Handler
pub struct ProcFilesystem
{
    mount_id: Option<usize>,
    vfs: Option<&'static mut crate::fs::vfs::FilesystemInterface>,
}

impl ProcFilesystem
{
    /// Create a new dev filesystem
    pub fn new() -> Self
    {
        Self
        {
            mount_id: None,
            vfs: None,
        }
    }
}

impl Filesystem for ProcFilesystem
{
    fn init(&mut self) -> FilesystemResult<()>
    {
        // Nothing needs to be done here
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
            if inode.inode == 1 || inode.inode & PROC_INODE_FLAG_PID > 0
            {
                let mut result = Vec::new();

                // Construct the loop back and parent entries
                let loopback = DirectoryEntry{
                    index: FilesystemIndex { mount_id: inode.mount_id, inode: 1},
                    name: String::from("."),
                    entry_type: DirectoryEntryType::Directory,
                };

                let parent = DirectoryEntry{
                    index: FilesystemIndex { mount_id: inode.mount_id, inode: inode.inode},
                    name: String::from(".."),
                    entry_type: DirectoryEntryType::Directory,
                };

                result.push(loopback);
                result.push(parent);

                if inode.inode == 1
                {
                    if let Some(proc_manager) = process::scheduler::get_process_manager()
                    {
                        for key in proc_manager.processes.keys()
                        {
                            let entry = DirectoryEntry{
                                index: FilesystemIndex { mount_id: inode.mount_id, inode: PROC_INODE_FLAG_PID | (*key as usize)},
                                name: format!("{}", *key),
                                entry_type: DirectoryEntryType::Directory,
                            };
            
                            result.push(entry);
                        }
                    }
                }
                else if inode.inode & PROC_INODE_FLAG_PID > 0
                {
                    let pid = inode.inode & 0xFFFF;

                    if let Some(proc_manager) = process::scheduler::get_process_manager()
                    {
                        if let Some(_) = proc_manager.get_process_by_pid(pid as u16)
                        {
                            let entry = DirectoryEntry
                                {
                                    index: FilesystemIndex { mount_id: inode.mount_id, inode: PROC_INODE_FLAG_PID_CMDLINE | (pid as usize)},
                                    name: String::from("cmdline"),
                                    entry_type: DirectoryEntryType::RegularFile,
                                };

                            result.push(entry);

                            let entry = DirectoryEntry
                                {
                                    index: FilesystemIndex { mount_id: inode.mount_id, inode: PROC_INODE_FLAG_PID_STATM | (pid as usize)},
                                    name: String::from("statm"),
                                    entry_type: DirectoryEntryType::RegularFile,
                                };

                            result.push(entry);
                        }
                    }
                }

                Ok(result)
            }
            else if inode.inode & !0xFFFF > 0
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
            let pid = inode.inode & 0xFFFF;

            if inode.inode & PROC_INODE_FLAG_PID_CMDLINE > 0
            {
                if let Some(proc_manager) = process::scheduler::get_process_manager()
                {
                    if let Some(proc) = proc_manager.get_process_by_pid(pid as u16)
                    {
                        Ok(Vec::from(proc.data.command_line_args_to_string().as_bytes()))
                    }
                    else
                    {
                        Err(FilesystemError::BadINode)
                    }
                }
                else
                {
                    Ok(Vec::new())
                }
            }
            else if inode.inode & PROC_INODE_FLAG_PID_STATM > 0
            {
                if let Some(proc_manager) = process::scheduler::get_process_manager()
                {
                    if let Some(proc) = proc_manager.get_process_by_pid(pid as u16)
                    {
                        Ok(Vec::from(proc.data.mem_stats.to_string().as_bytes()))
                    }
                    else
                    {
                        Err(FilesystemError::BadINode)
                    }
                }
                else
                {
                    Ok(Vec::new())
                }
            }
            else
            {
                Ok(Vec::new())
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
                if inode.inode == 1 || inode.inode & PROC_INODE_FLAG_PID > 0
                {
                    Ok(Box::new(InodeFileDescriptor::new(vfs, inode, mode).unwrap()))
                }
                else if inode.inode & (PROC_INODE_FLAG_PID_CMDLINE | PROC_INODE_FLAG_PID_STATM) > 0
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