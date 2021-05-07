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
    pub satp: usize
}

/// Schedule the next process to run
pub fn schedule_next(process_list: &mut ProcessManager) -> Result<ScheduleResult, ()>
{
    let start_pid = process_list.get_next_pid();
    let mut next_pid = start_pid;
    loop
    {
        if let Some(process) = process_list.process_by_pid(next_pid)
        {
            if process.is_running()
            {
                let frame_addr = process.get_frame_pointer();
                let mepc = process.get_program_counter();
                let satp = process.get_satp();

                break Ok(ScheduleResult{frame_addr, mepc, satp});
            }

            if process.is_dead()
            {
                process_list.remove_process(next_pid);
            }
        }

        next_pid = process_list.get_next_pid();

        if start_pid == next_pid
        {
            panic!("No running processes!");
        }
    }
}

/// Trigger a process switch
pub fn process_switch() -> !
{
    let result = schedule_next(super::get_process_manager().unwrap()).unwrap();

    unsafe { switch_to_user(result.frame_addr, result.mepc, result.satp) };
}