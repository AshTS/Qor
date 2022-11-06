use super::*;

/// Process Interface
pub struct ProcessInterface {
    inner: core::cell::UnsafeCell<Process>,
}

impl ProcessInterface {
    /// Get the PID of the process
    pub fn pid(&self) -> ProcessIdentifier {
        unsafe { self.inner.get().as_mut().unwrap() }.const_data.pid
    }

    /// Get the status of the process
    pub fn status(&self) -> ProcessState {
        unsafe { self.inner.get().as_mut().unwrap() }
            .atomic_data
            .status
            .load(core::sync::atomic::Ordering::AcqRel)
    }
}

// A process interface is allowed to be shared between threads, this is because
// it is only a handle to the actual process, which will have access to the
// dangerous parts mediated by a mutex
unsafe impl Send for ProcessInterface {}
unsafe impl Sync for ProcessInterface {}
