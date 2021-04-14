use crate::process::manager::ProcessManager;

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
        }

        next_pid = process_list.get_next_pid();

        if start_pid == next_pid
        {
            panic!("No running processes!");
        }
    }
}