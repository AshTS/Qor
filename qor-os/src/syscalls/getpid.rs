/// getpid Syscall
pub fn syscall_getpid(proc: &mut super::Process) -> usize
{
    proc.pid as usize
}