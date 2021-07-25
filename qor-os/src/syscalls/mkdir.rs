use crate::*;

use fs::fstrait::Filesystem;
use libutils::paths::OwnedPath;

/// mkdir Syscall
pub fn syscall_mkdir(proc: &mut super::Process, path_ptr: usize, _mode: usize) -> usize
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

    // Expand the path
    let mut expanded = OwnedPath::new(path);
    expanded.canonicalize(&proc.data.cwd);


    let (dest_path, name) = expanded.split_last();

    let vfs = crate::fs::vfs::get_vfs_reference().unwrap();

    if let Ok(dest_inode) = vfs.path_to_inode(&dest_path)
    {
        if let Ok(_) = vfs.create_directory(dest_inode, name.to_string())
        {
            return 0;
        }
    }

    usize::MAX
}