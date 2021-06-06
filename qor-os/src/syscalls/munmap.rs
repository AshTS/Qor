/// munmap Syscall
pub fn syscall_munmap(proc: &mut super::Process, start_ptr: usize, length: usize) -> usize
{
    proc.unmap(start_ptr, length)
}