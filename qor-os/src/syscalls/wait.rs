use crate::*;

/// Wait Syscall
pub fn syscall_wait(proc: &mut super::Process, ptr: usize) -> usize
{
    let _status = 
        if ptr != 0
        {
            proc.map_mem(ptr).unwrap() as *mut u32
        }
        else
        {
            0 as *mut u32
        };

    proc.state = process::process::ProcessState::Waiting(process::process::WaitMode::ForChild);
    proc.program_counter += 4;

    let schedule = process::scheduler::schedule_next();
    process::scheduler::schedule_jump(schedule);
}