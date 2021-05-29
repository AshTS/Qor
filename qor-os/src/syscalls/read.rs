/// Read Syscall
pub fn syscall_read(proc: &mut super::Process, fd: usize, buffer: usize, count: usize) -> usize
{
    let ptr = proc.map_mem(buffer).unwrap() as *mut u8;

    proc.read(fd, ptr, count)
}