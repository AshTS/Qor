use libutils::sync::MutexGuard;

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

    /// Obtain a lock on the mutable data for the process
    pub fn lock_mutable(&self) -> MutexGuard<'_, MutableProcessData> {
        self.inner.lock_mutable()
    }
}
