use crate::*;

/// Dup Syscall
pub fn syscall_dup(proc: &mut super::Process, old_fd: usize)-> usize
{
    kdebugln!(Syscalls, "Duplicating FD {}on Process PID {}", old_fd, proc.pid);

    proc.dup(old_fd, None)
}

/// Dup2 Syscall
pub fn syscall_dup2(proc: &mut super::Process, old_fd: usize, new_fd: usize)-> usize
{
    kdebugln!(Syscalls, "Duplicating FD {} to {} on Process PID {}", old_fd, new_fd, proc.pid);

    proc.dup(old_fd, Some(new_fd))
}