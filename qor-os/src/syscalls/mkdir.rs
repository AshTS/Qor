use crate::*;

use fs::fstrait::Filesystem;

/// mkdir Syscall
pub fn syscall_mkdir(proc: &mut super::Process, path_ptr: usize, _mode: usize) -> Result<usize, usize>
{
    let expanded = super::utils::userspace_string_to_path(proc, path_ptr)?;
    let (dest_path, name) = expanded.split_last();

    let vfs = crate::fs::vfs::get_vfs_reference().unwrap();

    if let Ok(dest_inode) = vfs.path_to_inode(&dest_path)
    {
        if let Ok(_) = vfs.create_directory(dest_inode, name.to_string())
        {
            return Ok(0);
        }
    }

    Err(usize::MAX)
}