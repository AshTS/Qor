use super::{ProcessIdentifier, WaitReason};

/// Schedule the next process and switch to it, or return if no process is ready or it is unable to access the process map
pub fn schedule(hart: usize) {
    let mut scheduled_pid = None;

    if let Some(proc_map) = crate::process::process_map().attempt_shared() {
        for (pid, interface) in proc_map.iter() {
            if let Some(mut state) = interface.state_mutex().attempt_lock() {
                crate::kdebugln!(unsafe "Schedule Time {} {:?}", pid, *state);
                
                match *state {
                    // If a process is pending, it is ready to run now
                    super::ProcessState::Pending => {
                        scheduled_pid = Some(*pid);
                        *state = super::ProcessState::Running;
                        break;
                    }
                    
                    // If a process is waiting for its children, check to see if it has had children die
                    super::ProcessState::Waiting(WaitReason::ForChildren) => {
                        if interface.check_child_semaphore() {
                            scheduled_pid = Some(*pid);
                            *state = super::ProcessState::Running;
                            break;
                        }
                    }

                    super::ProcessState::Waiting(WaitReason::Semaphore) => {
                        if interface.check_wait_semaphore() == Some(true) {
                            scheduled_pid = Some(*pid);
                            *state = super::ProcessState::Running;
                            break;
                        }
                    }

                    // A zombie process needs to notify its parent and then can be marked for clean up
                    super::ProcessState::Zombie => {
                        interface.set_state(super::ProcessState::Dead);
                    },

                    // If a processor is dead, we request it to be cleaned up
                    super::ProcessState::Dead => {
                        crate::tasks::add_global_executor_task(clean_up_pid(*pid));
                    },
                    // If a process is running, we can ignore it
                    super::ProcessState::Running => {},
                }
            }
            else {
                crate::kdebugln!(unsafe "Schedule Time {} Unable to get State", pid);}
        }
    }

    // If a context switch has been requested, do the context switch
    if let Some(pid) = scheduled_pid {
        if let Some(proc) = super::get_process(pid) {
            crate::drivers::CLINT_DRIVER.set_remaining(hart, 10_000_000);
            unsafe { proc.switch_to() };
        }
    }
}

/// Clean up a process with the given pid
pub async fn clean_up_pid(pid: ProcessIdentifier) {
    crate::process::process_map().async_unique().await.remove(&pid);
}