use crate::*;

use fs::fstrait::Filesystem;
use libutils::paths::OwnedPath;

/// chdir Syscall
pub fn syscall_chdir(proc: &mut super::Process, path_ptr: usize) -> Result<usize, usize>
{
    let path = super::utils::userspace_string_to_path(proc, path_ptr)?;
    
    proc.ensure_fs();

    if let Ok(inode) = proc.fs_interface.as_mut().unwrap().path_to_inode(&path)
    {
        if let Ok(path) = proc.fs_interface.as_mut().unwrap().inode_to_path(inode)
        {
            if path.as_str().len() == 0
            {
                proc.data.cwd = OwnedPath::new("/");
            }
            else
            {
                proc.data.cwd = path.clone();
            }

            if !proc.data.cwd.as_str().ends_with("/")
            {
                proc.data.cwd.as_mut_str().push('/');
            }

            Ok(0)
        }
        else
        {
            Err(errno::EIO)
        }
    }
    else
    {
        Err(errno::ENOENT) // No entry - Directory not found
    }
}