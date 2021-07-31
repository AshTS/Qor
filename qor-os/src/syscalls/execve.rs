use crate::*;

use alloc::format;
use libutils::paths::OwnedPath;

/// Execve Syscall
pub fn syscall_execve(proc: &mut super::Process, path_ptr: usize, argv_ptr: usize, envp_ptr: usize) -> usize
{
    let path_ptr = proc.map_mem(path_ptr).unwrap() as *mut u8;
    let argv_ptr = proc.map_mem(argv_ptr).unwrap() as *mut usize;
    let envp_ptr = proc.map_mem(envp_ptr).unwrap() as *mut usize;

    // Convert argv_ptr to a vector of &[u8]
    let mut argv_vecs = Vec::new();

    for i in 0..
    {
        // Get the pointer at the given value
        let ptr = unsafe { argv_ptr.add(i).read() };   
        if ptr == 0 { break };

        // Convert the pointer to a physical address
        let ptr = proc.map_mem(ptr).unwrap() as *mut u8;

        let mut current_vec = Vec::new();

        for j in 0..
        {
            let v = unsafe { ptr.add(j).read() };
            current_vec.push(v);
            if v == 0 { break };
        }

        argv_vecs.push(current_vec);
    }

    let mut argv_vals = Vec::with_capacity(argv_vecs.len());

    for v in &argv_vecs
    {
        argv_vals.push(v.as_slice());
    }

    
    // Convert envp_ptr to a vector of &[u8]
    let mut envp_vecs = Vec::new();

    for i in 0..
    {
        // Get the pointer at the given value
        let ptr = unsafe { envp_ptr.add(i).read() };   
        if ptr == 0 { break };

        // Convert the pointer to a physical address
        let ptr = proc.map_mem(ptr).unwrap() as *mut u8;

        let mut current_vec = Vec::new();

        for j in 0..
        {
            let v = unsafe { ptr.add(j).read() };
            current_vec.push(v);
            if v == 0 { break };
        }

        envp_vecs.push(current_vec);
    }

    let mut envp_vals = Vec::with_capacity(envp_vecs.len());

    for v in &envp_vecs
    {
        envp_vals.push(v.as_slice());
    }

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

    if !path.starts_with("/")
    {
        path = format!("{}{}", proc.data.cwd, path);
    }
     
    // Create a process from an elf file
    if let Ok(mut new_proc) = process::elf::load_elf(proc.fs_interface.as_mut().unwrap(), &OwnedPath::new(path))
    // if true
    {
        new_proc.data.descriptors = proc.data.descriptors.clone();

        new_proc.data.cwd = proc.data.cwd.clone();

        new_proc.set_arguments(argv_vals.as_slice(), envp_vals.as_slice());

        process::scheduler::replace_process(proc.pid, new_proc);
        
        let schedule = process::scheduler::schedule_next();
        process::scheduler::schedule_jump(schedule);
    }
    else
    {
        0xFFFFFFFFFFFFFFFF
    }
}