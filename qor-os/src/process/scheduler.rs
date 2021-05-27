use core::u16;

use crate::*;

use super::process::Process;
use super::process::ProcessState;

use alloc::collections::BTreeMap;

static mut GLOBAL_PROC_MANAGER: Option<ProcessManager> = None;

/// Process Manager
pub struct ProcessManager
{
    current_pid: Option<u16>,
    max_pid: Option<u16>,
    processes: BTreeMap<u16, Process>
}

impl ProcessManager
{
    /// Create a new process manager
    pub fn new() -> Self
    {
        Self
        {
            current_pid: None,
            max_pid: None,
            processes: BTreeMap::new()
        }
    }

    /// Add a process
    pub fn add_process(&mut self, proc: Process)
    {
        kdebugln!(Processes, "Adding process with PID {}", proc.pid);

        if let Some(old_max) = self.max_pid
        {
            self.max_pid = Some(old_max.max(proc.pid));
        }
        else
        {
            self.max_pid = Some(proc.pid);
        }

        self.processes.insert(proc.pid, proc);
    }

    /// Get a reference to a process by pid
    pub fn get_process_by_pid(&self, pid: u16) -> Option<&Process>
    {
        self.processes.get(&pid)
    }

    /// Get a mutable reference to a process by pid
    pub fn get_process_by_pid_mut(&mut self, pid: u16) -> Option<&mut Process>
    {
        self.processes.get_mut(&pid)
    }

    /// Get a reference to the currently running process
    pub fn currently_running(&self) -> Option<&Process>
    {
        if let Some(pid) = self.current_pid
        {
            self.get_process_by_pid(pid)
        }
        else
        {
            None
        }
    }
    
    /// Get a mutable reference to the currently running process
    pub fn currently_running_mut(&mut self) -> Option<&mut Process>
    {
        if let Some(pid) = self.current_pid
        {
            self.get_process_by_pid_mut(pid)
        }
        else
        {
            None
        }
    }

    /// Schedule the next process
    pub fn schedule_process(&mut self) -> (usize, usize, usize)
    {
        let next_pid = self.pid_of_next();

        self.current_pid = Some(next_pid);

        kdebugln!(Scheduling, "Scheduling PID {}", next_pid);

        self.get_schedule_info(next_pid)
    }

    /// Schedule the next process by returning a pid
    pub fn pid_of_next(&mut self) -> u16
    {
        if let Some(pid) = self.current_pid
        {
            let highest = self.max_pid.unwrap();

            let mut step_pid = pid;

            loop
            {
                // Increment and wrap back to zero
                step_pid = (step_pid + 1) % (highest + 1);

                // Check the current step_pid
                if let Some(proc) = self.get_process_by_pid(step_pid)
                {
                    match proc.get_state()
                    {
                        // If the process is running, switch to it
                        ProcessState::Running => 
                        {
                            break;
                        },
                        // If it is asleep or waiting, skip
                        ProcessState::Sleeping | ProcessState::Waiting => {},
                        // If it is dead, remove it from the process tree
                        ProcessState::Dead => 
                        {
                            self.processes.remove(&step_pid);
                        }
                    }
                }

                // If the step wraps back to the original pid, panic
                if step_pid == pid
                {
                    panic!("No processes remaining");
                }
            }

            step_pid
        }
        // If this is the first scheduling, schedule the init process
        else
        {
            0
        }
    }

    /// Get the scheduling information for the given pid
    fn get_schedule_info(&self, pid: u16) -> (usize, usize, usize)
    {
        if let Some(proc) = self.processes.get(&pid)
        {
            (&proc.frame as *const trap::TrapFrame as usize, proc.program_counter, (8 << 60) | (proc.root as usize >> 12))
        }
        else
        {
            (0, 0, 0)
        }
    }
}

/// Initialize a process manager
pub fn init_process_manager()
{
    unsafe 
    {
        GLOBAL_PROC_MANAGER = Some(ProcessManager::new());
    }
}

/// Add a process to the global process manager
pub fn add_process(proc: Process)
{
    unsafe 
    {
        GLOBAL_PROC_MANAGER.as_mut().unwrap().add_process(proc);
    }
}

/// Get the current process
pub fn current_process() -> Option<&'static mut Process>
{
    unsafe 
    {
        GLOBAL_PROC_MANAGER.as_mut().unwrap().currently_running_mut()
    }
}

/// Schedule the next process
pub fn schedule_next() -> (usize, usize, usize)
{
    unsafe 
    {
        GLOBAL_PROC_MANAGER.as_mut().unwrap().schedule_process()
    }
}

extern "C"
{
    pub fn switch_to_user(frame: usize, pc: usize, satp: usize) -> !;
}

/// Jump into the process
pub fn schedule_jump(data: (usize, usize, usize)) -> !
{
    unsafe { switch_to_user(data.0, data.1, data.2) }
}