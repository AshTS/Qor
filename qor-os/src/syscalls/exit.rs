use crate::*;

/// Exit Syscall
pub fn syscall_exit(proc: &mut super::Process, value: usize)
{
    kdebugln!(Syscalls, "Exiting Process PID {} with value: {}", proc.pid, value);

    proc.kill(value);

    let schedule = process::scheduler::schedule_next();
    process::scheduler::schedule_jump(schedule);
}