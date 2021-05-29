/// Write Syscall
pub fn syscall_write(proc: &mut super::Process, fd: usize, buffer: usize, count: usize) -> usize
{
    let ptr = proc.map_mem(buffer).unwrap() as *mut u8;

    proc.write(fd, ptr, count)
}