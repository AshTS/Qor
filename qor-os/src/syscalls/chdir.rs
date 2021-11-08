use crate::*;

use fs::fstrait::Filesystem;
use libutils::paths::OwnedPath;

/// chdir Syscall
pub fn syscall_chdir(proc: &mut super::Process, path_ptr: usize) -> usize
{
    let path_ptr = proc.map_mem(path_ptr).unwrap() as *mut u8;
    let mut path = String::new();

    let mut i = 0; 

    loop
    {
        let v = unsafe { path_ptr.add(i).read() } as char;

        if v == '\x00' { break; }

        path.push(v);

        i += 1;
    }

    let path = OwnedPath::new(path).canonicalized(&proc.data.cwd);
    
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

            0
        }
        else
        {
            errno::EFAULT // Path points outside of addressable area
        }
    }
    else
    {
        errno::ENOENT // No entry - Directory not found
    }
}