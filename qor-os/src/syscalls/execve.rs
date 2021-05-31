use crate::*;

/// Execve Syscall
pub fn syscall_execve(proc: &mut super::Process, path_ptr: usize) -> usize
{
    let path_ptr = proc.map_mem(path_ptr).unwrap() as *mut u8;

    // Ensure the filesystem has been initialized
    proc.ensure_fs();

    let mut path = String::new();

    let mut i = 0; 

    loop
    {
        let v = unsafe { path_ptr.add(i).read() } as char;

        if v == '\x00' { break; }

        path.push(v);

        i += 1;
    }
     
    // Create a process from an elf file
    if let Ok(new_proc) = process::elf::load_elf(proc.fs_interface.as_mut().unwrap(), &path)
    // if true
    {
        let pid = new_proc.pid;
        process::scheduler::add_process(new_proc);
        // 0

        pid as usize
    }
    else
    {

        0xFFFFFFFFFFFFFFFF
    }
}