use crate::*;

/// Pause Syscall
pub fn syscall_pause(proc: &mut super::Process) -> usize
{
    proc.state = process::process::ProcessState::Waiting(process::process::WaitMode::ForSignal);
    proc.program_counter += 4;

    let schedule = process::scheduler::schedule_next();
    process::scheduler::schedule_jump(schedule);
}