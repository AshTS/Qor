use crate::*;

use super::process::Process;
use super::process::ProcessState;
use super::signals::POSIXSignal;

use alloc::collections::BTreeMap;

static mut GLOBAL_PROC_MANAGER: Option<ProcessManager> = None;

use super::PID;

/// Process Manager
pub struct ProcessManager
{
    current_pid: Option<PID>,
    max_pid: Option<PID>,
    pub processes: BTreeMap<PID, Box<Process>>
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

        self.processes.insert(proc.pid, Box::new(proc));
    }

    /// Replace a process
    pub fn replace_process(&mut self, pid: PID, mut proc: Process)
    {
        kdebugln!(Processes, "Replacing process with PID {}", pid);
        assert!(self.processes.contains_key(&pid));

        proc.pid = pid;

        self.processes.insert(pid, Box::new(proc));
    }

    /// Get a reference to a process by pid
    pub fn get_process_by_pid(&self, pid: PID) -> Option<&Box<Process>>
    {
        self.processes.get(&pid)
    }

    /// Get a mutable reference to a process by pid
    pub fn get_process_by_pid_mut(&mut self, pid: PID) -> Option<&mut Box<Process>>
    {
        self.processes.get_mut(&pid)
    }

    /// Get a reference to the currently running process
    pub fn currently_running(&self) -> Option<&Box<Process>>
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
    pub fn currently_running_mut(&mut self) -> Option<&mut Box<Process>>
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

        self.schedule_pid(next_pid)
    }

    /// Schedule the given pid
    pub fn schedule_pid(&mut self, pid: PID) -> (usize, usize, usize)
    {
        self.current_pid = Some(pid);

        kdebugln!(Scheduling, "Scheduling PID {}", pid);

        self.get_schedule_info(pid)
    }

    /// Schedule the next process by returning a pid
    pub fn pid_of_next(&mut self) -> PID
    {
        if let Some(pid) = self.current_pid
        {
            let highest = self.max_pid.unwrap();

            let mut step_pid = pid;

            let mut stopped_pid = None;

            loop
            {
                // Increment and wrap back to zero
                step_pid = (step_pid + 1) % (highest + 1);

                let mut children = None;
                let mut adoption_data: Option<(PID, Vec<PID>)> = None;

                // Check the current step_pid
                if let Some(proc) = self.get_process_by_pid_mut(step_pid)
                {
                    if proc.get_state() != ProcessState::Dead && proc.get_state() != ProcessState::Zombie
                    {
                        if let Some(sig) = proc.pop_signal()
                        {
                            if proc.get_state() == ProcessState::Waiting(process::process::WaitMode::ForSignal)
                            {
                                proc.state = ProcessState::Running;
                            }

                            if proc.trigger_signal(sig)
                            {
                                return proc.pid;
                            }
                        }
                    }

                    match proc.get_state()
                    {
                        // If the process is running, switch to it
                        ProcessState::Running => 
                        {
                            break;
                        },
                        // If the process is waiting, perform the proper wait checks
                        ProcessState::Waiting(mode) =>
                        {
                            match mode
                            {
                                process::process::WaitMode::ForChild => 
                                {
                                    children = Some(proc.get_children().clone())
                                },
                                process::process::WaitMode::ForSignal => {},
                            }
                            
                        },
                        // If it is asleep, check if it hsould be woken up
                        ProcessState::Sleeping {wake_time } =>
                        {
                            // Check if the wake time has passed by checking the timer driver
                            let current = unsafe { &drivers::TIMER_DRIVER }.time();

                            // If so, switch the process to the Running state and switch to it
                            if current > wake_time
                            {
                                break;
                            }
                        },
                        // If the process is a zombie or stopped, ignore it
                        ProcessState::Zombie | ProcessState::Stopped => {},
                        // If it is dead, remove it from the process tree
                        ProcessState::Dead => 
                        {
                            kdebugln!(Processes, "Cleaning Up PID {}", step_pid);
                            adoption_data = Some((proc.data.parent_pid, proc.data.children.clone()));
                            self.processes.remove(&step_pid);
                        }
                    }
                }

                // If this process is waiting
                if let Some(children) = children
                {
                    for child in children
                    {
                        if let Some(child_proc) = self.get_process_by_pid_mut(child)
                        {
                            if child_proc.wait_check()
                            {
                                stopped_pid = Some(child);
                                break;
                            }
                        }
                    }

                    if let Some(stopped) = stopped_pid
                    {
                        self.get_process_by_pid_mut(step_pid).unwrap().remove_child(stopped);
                        unsafe { self.get_process_by_pid_mut(step_pid).unwrap().frame.as_mut().unwrap() }.regs[10] = stopped as usize;
                        self.get_process_by_pid_mut(step_pid).unwrap().state = ProcessState::Running;
                        break;
                    }
                }

                // If data needs to be adopted
                if let Some((pid, data)) = adoption_data
                {
                    if let Some(r) = self.get_process_by_pid_mut(pid)
                    {
                        for cpid in data
                        {
                            if !r.data.children.contains(&cpid)
                            {
                                r.register_child(cpid);
                            }
                        }
                    }
                }
            }

            // If the process was woken up or sleeping, make sure it is running now
            self.get_process_by_pid_mut(step_pid).unwrap().state = ProcessState::Running;

            step_pid
        }
        // If this is the first scheduling, schedule the init process
        else
        {
            // Ensure the 0 process exists, otherwise panic
            if !self.processes.contains_key(&0)
            {
                panic!("No Processes Initialized");
            }
            0
        }
    }

    /// Get the scheduling information for the given pid
    fn get_schedule_info(&self, pid: PID) -> (usize, usize, usize)
    {
        if let Some(proc) = self.processes.get(&pid)
        {
            let trap_frame = proc.frame as usize;

            (trap_frame, proc.program_counter, (8 << 60) | ((pid as usize) << 44) | (proc.root as usize >> 12))
        }
        else
        {
            (0, 0, 0)
        }
    }

    /// Send a signal between processes
    pub fn send_signal(&mut self, dest_pid: PID, signal: POSIXSignal) -> Result<(), ()>
    {
        kdebugln!(Signals, "Sending Signal {:?} to PID {}", signal.sig_type, dest_pid);

        if let Some(proc) = self.get_process_by_pid_mut(dest_pid)
        {
            proc.push_signal(signal)?;
        }

        Ok(())
    }
}

/// Initialize a process manager
pub fn init_process_manager()
{
    unsafe 
    {
        GLOBAL_PROC_MANAGER = Some(ProcessManager::new());
    }

    // Add the init process
    let process = super::process::Process::from_fn_ptr(super::init::init_proc);
    add_process(process);
}

/// Add a process to the global process manager
pub fn add_process(proc: Process)
{
    unsafe 
    {
        GLOBAL_PROC_MANAGER.as_mut().unwrap().add_process(proc);
    }
}

/// Get a reference to the init process
pub fn get_init_process() -> Option<&'static Box<Process>>
{
    unsafe 
    {
        GLOBAL_PROC_MANAGER.as_mut().unwrap().get_process_by_pid(0)
    }
}

/// Get a mutable reference to the init process
pub fn get_init_process_mut() -> Option<&'static mut Box<Process>>
{
    unsafe 
    {
        GLOBAL_PROC_MANAGER.as_mut().unwrap().get_process_by_pid_mut(0)
    }
}

/// Replace a running process
pub fn replace_process(pid: PID, proc: Process)
{
    unsafe 
    {
        GLOBAL_PROC_MANAGER.as_mut().unwrap().replace_process(pid, proc);
    }
}


/// Get the current process
pub fn current_process() -> Option<&'static mut Box<Process>>
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

/// Get a reference to the process manager
pub fn get_process_manager() -> Option<&'static mut ProcessManager>
{
    if let Some(data) = unsafe { &mut GLOBAL_PROC_MANAGER }
    {
        Some(data)
    }
    else
    {
        None
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