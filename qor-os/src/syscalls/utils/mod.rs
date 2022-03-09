use crate::*;
use super::Process;
use libutils::paths::OwnedPath;

// Constants for error handling with long paths
pub const MAX_PATH_LENGTH: usize = 128;

/// Convert a userspace string into a canonicalized path
pub fn userspace_string_to_path(proc: &mut Process, userspace_ptr: usize) -> Result<OwnedPath, usize>
{
    let path_ptr = proc.map_mem(userspace_ptr).map_err( |_| errno::EFAULT )? as *mut u8;
    let mut path = String::new();

    let mut i = 0; 

    loop
    {
        if i > MAX_PATH_LENGTH
        {
            return Err(errno::ENAMETOOLONG);
        }

        let v = unsafe { path_ptr.add(i).read() } as char;

        if v == '\x00' { break; }

        path.push(v);

        i += 1;
    }

    let mut expanded_path = OwnedPath::new(path);
    expanded_path.canonicalize(&proc.data.cwd);

    Ok(expanded_path)
}