use crate::fs::fstrait::*;
use crate::fs::structures::*;

/// Minix3 Filesystem Driver
pub struct Minix3Filesystem
{
}

impl Filesystem for Minix3Filesystem
{
    fn init(&mut self)
    {
        todo!()
    }

    fn sync(&mut self) -> FilesystemResult<()>
    {
        todo!()
    }

    fn set_mount_id(&mut self, mount_id: usize, vfs: &'static mut crate::fs::vfs::FilesystemInterface)
    {
        todo!()
    }

    fn get_root_index(&mut self) -> FilesystemResult<FilesystemIndex>
    {
        todo!()
    }

    fn path_to_inode(&mut self, path: &str) -> FilesystemResult<FilesystemIndex>
    {
        todo!()
    }

    fn inode_to_path(&mut self, inode: FilesystemIndex) -> FilesystemResult<&str>
    {
        todo!()
    }

    fn get_dir_entries(&mut self, inode: FilesystemIndex) -> FilesystemResult<alloc::vec::Vec<DirectoryEntry>>
    {
        todo!()
    }

    fn create_file(&mut self, inode: FilesystemIndex, name: alloc::string::String) -> FilesystemResult<FilesystemIndex>
    {
        todo!()
    }

    fn create_directory(&mut self, inode: FilesystemIndex, name: alloc::string::String) -> FilesystemResult<FilesystemIndex>
    {
        todo!()
    }

    fn remove_inode(&mut self, inode: FilesystemIndex, directory: FilesystemIndex) -> FilesystemResult<()>
    {
        todo!()
    }
}

