use crate::*;

/// Exit Syscall
pub fn syscall_exit(proc: &mut super::Process, value: usize)
{
    kdebugln!(Syscalls, "Exiting Process PID {} with value: {}", proc.pid, value);

    proc.kill(value);
}