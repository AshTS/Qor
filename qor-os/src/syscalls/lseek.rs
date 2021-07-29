/// lseek Syscall
pub fn syscall_lseek(proc: &mut super::Process, fd: usize, offset: usize, mode: usize) -> usize
{
    proc.seek(fd, offset, mode)
}