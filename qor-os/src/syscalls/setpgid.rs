use crate::process::PID;

/// Pause setpgid
pub fn syscall_setpgid(proc: &mut super::Process, pid: usize, pgid: usize) -> usize
{
    if proc.pid == pid as PID
    {
        proc.data.process_group_id = pgid as PID;
        return 0;
    }

    usize::MAX
}