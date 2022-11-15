use super::{ProcessIdentifier, WaitReason};

/// Schedule the next process and switch to it, or return if no process is ready or it is unable to access the process map
pub fn schedule() {
    if let Some(proc_map) = crate::process::process_map().attempt_shared() {
        for (pid, interface) in proc_map.iter() {
            match interface.state() {
                // If a process is pending, it is ready to run now
                super::ProcessState::Pending => {
                    unsafe { interface.switch_to() };
                }
                
                // If a process is waiting for its children, check to see if it has had children die
                super::ProcessState::Waiting(WaitReason::ForChildren) => {
                    if interface.check_child_semaphore() {
                        unsafe { interface.switch_to() };
                    }
                }

                super::ProcessState::Waiting(WaitReason::Semaphore) => {
                    if interface.check_wait_semaphore() == Some(true) {
                        unsafe { interface.switch_to() };
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
    }
}

/// Clean up a process with the given pid
pub async fn clean_up_pid(pid: ProcessIdentifier) {
    crate::process::process_map().async_unique().await.remove(&pid);
}