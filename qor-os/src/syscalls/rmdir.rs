use libutils::paths::OwnedPath;

use crate::*;

/// rmdir Syscall
pub fn syscall_rmdir(proc: &mut super::Process, path_ptr: usize) -> usize
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

    let mut expanded_path = OwnedPath::new(path);
    expanded_path.canonicalize(&proc.data.cwd);

    if let Err(e) = proc.rmdir(expanded_path)
    {
        return e;
    }

    0
}