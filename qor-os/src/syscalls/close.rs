/// Close Syscall
pub fn syscall_close(proc: &mut super::Process, fd: usize) -> usize
{
    proc.close(fd)
}