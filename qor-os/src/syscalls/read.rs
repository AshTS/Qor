use crate::{errno, process::process::{ProcessState, WaitMode}};
use crate::process;

/// Read Syscall
pub fn syscall_read(proc: &mut super::Process, fd: usize, buffer: usize, count: usize) -> usize
{
    let ptr = proc.map_mem(buffer).unwrap() as *mut u8;

    if !proc.data.descriptors.contains_key(&fd)
    {
        return errno::EBADFD;
    }

    if proc.check_available(fd)
    {
        proc.read(fd, ptr, count)
    }
    else
    {
        proc.state = ProcessState::Waiting(WaitMode::ForIO((fd, count, ptr)));
        proc.program_counter += 4;
        
        let schedule = process::scheduler::schedule_next();
        process::scheduler::schedule_jump(schedule);
    }
}