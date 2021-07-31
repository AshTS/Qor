/// Pipe Syscall
pub fn syscall_pipe(proc: &mut super::Process, fds: usize) -> usize
{
    let buffer = proc.map_mem(fds).unwrap() as *mut u32;

    let (read, write) = proc.pipe();
    
    unsafe
    {
        buffer.add(0).write(read as u32);
        buffer.add(1).write(write as u32);
    }

    0
}