use crate::*;

/// Fork Syscall
pub fn syscall_fork(proc: &mut super::Process) -> usize
{
    // Get the forked process
    let forked = proc.forked();

    let pid = forked.pid;

    process::scheduler::add_process(forked);

    pid as usize
}