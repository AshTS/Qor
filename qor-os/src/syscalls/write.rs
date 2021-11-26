use crate::*;

/// Write Syscall
pub fn syscall_write(proc: &mut super::Process, fd: usize, buffer: usize, count: usize) -> usize
{
    if (buffer < 0x1_0000_0000)
    {
        // kdebugln!("PID {} : fd: {} Buffer: {:x} count: {}", proc.pid, fd, buffer, count);
    }

    let ptr = proc.map_mem(buffer).unwrap() as *mut u8;

    proc.write(fd, ptr, count)
}