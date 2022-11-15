use libutils::sync::{MutexGuard, semaphore::SignalSemaphoreSender};

use super::*;

/// Process Interface
#[derive(Clone)]
pub struct ProcessInterface {
    inner: alloc::sync::Arc<Process>,
}

impl ProcessInterface {
    /// Construct a new process interface
    pub fn new(inner: alloc::sync::Arc<Process>) -> Self {
        Self { inner }
    }

    /// Switch to this process
    pub unsafe fn switch_to(&self) -> ! {
        self.set_state(ProcessState::Running);
        self.inner.switch_to_process()
    }

    /// Get the PID of the process
    pub fn pid(&self) -> ProcessIdentifier {
        self.inner.pid()
    }

    /// Get the state of the process
    pub fn state(&self) -> ProcessState {
        self.inner.state()
    }

    /// Set the state of the process
    pub fn set_state(&self, state: ProcessState) {
        self.inner.set_state(state)
    }

    /// Obtain a lock on the mutable data for the process
    pub fn lock_mutable(&self) -> MutexGuard<'_, MutableProcessData> {
        self.inner.lock_mutable()
    }

    /// Check the child pending semaphore
    pub fn check_child_semaphore(&self) -> bool {
        self.inner.check_child_semaphore()
    }

    /// Check the optional waiting semaphore
    pub fn check_wait_semaphore(&self) -> Option<bool> {
        self.inner.check_wait_semaphore()
    }

    /// Get a new sender for the wait semaphore
    pub fn new_wait_semaphore(&self) -> SignalSemaphoreSender {
        self.inner.new_wait_semaphore()
    }
}
