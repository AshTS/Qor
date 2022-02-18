use crate::*;

/// Wait Syscall
pub fn syscall_wait(proc: &mut super::Process, ptr: usize) -> usize
{
    let status = 
        if ptr != 0
        {
            proc.map_mem(ptr).unwrap() as *mut u32
        }
        else
        {
            0 as *mut u32
        };

    if proc.data.children.len() == 0
    {
        return errno::ECHILD;
    }

    proc.state = process::process::ProcessState::Waiting(process::process::WaitMode::ForChild);
    proc.data.return_code_listener = unsafe { status.as_mut() };
    proc.program_counter += 4;

    let schedule = process::scheduler::schedule_next();
    process::scheduler::schedule_jump(schedule);
}