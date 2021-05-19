use crate::process::manager::ProcessManager;

// Bring in assembly function
extern "C"
{
    /// Switch over into user mode
	fn switch_to_user(frame: usize, mepc: usize, satp: usize) -> !;
}

/// Schedule result structure
#[derive(Debug, Clone, Copy)]
pub struct ScheduleResult
{
    pub frame_addr: usize,
    pub mepc: usize,
    pub satp: usize,
    pub pid: u16
}

/// Schedule the next process to run
pub fn schedule_next(process_list: &mut ProcessManager) -> Result<ScheduleResult, ()>
{
    if let Some(pid) = process_list.next_scheduled_pid()
    {
        if let Some(process) = process_list.process_by_pid(pid)
        {
            let frame_addr = process.get_frame_pointer();
            let mepc = process.get_program_counter();
            let satp = process.get_satp();
            
            return Ok(ScheduleResult{frame_addr, mepc, satp, pid});
        }
        else
        {
            panic!("Scheduled process does not exist (PID {})", pid);
        }
    }
    else
    {
        panic!("No running processes!");
    };
}
/// Trigger a process switch
pub fn process_switch() -> !
{
    let result = schedule_next(super::get_process_manager().unwrap()).unwrap();

    // Set next switch to trigger in 10 ms
    crate::drivers::TIMER_DRIVER.set_remaining_time(10000);

    unsafe { switch_to_user(result.frame_addr, result.mepc, result.satp) };
}