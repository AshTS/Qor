use crate::*;

/// Open Syscall
pub fn syscall_open(proc: &mut super::Process, path_ptr: usize, flags: usize) -> usize
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

    match proc.open(&path, flags)
    {
        Ok(v) => v,
        Err(_) => 0xFFFFFFFFFFFFFFFF
    }
}