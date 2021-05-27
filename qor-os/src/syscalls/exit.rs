use crate::*;

/// Exit Syscall
pub fn syscall_exit(proc: &mut super::Process, value: usize)
{
    kprintln!("Exiting Process PID {} with value: {}", proc.pid, value);

    proc.kill(value);
}