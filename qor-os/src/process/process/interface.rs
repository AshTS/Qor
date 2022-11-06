use super::*;

/// Process Interface
pub struct ProcessInterface {
    inner: core::cell::UnsafeCell<Process>,
}

impl ProcessInterface {
    /// Get the PID of the process
    pub fn pid(&self) -> ProcessIdentifier {
        // Safety: The data in the const_data section of the process structure
        // is constant and thus allowed to be read non-atomically at any time.
        // Furthermore, the pointer within the unsafe cell should not alias,
        // and will only be dropped when there are no more references to it.
        unsafe { self.inner.get().as_mut().unwrap() }.const_data.pid
    }

    /// Get the status of the process
    pub fn status(&self) -> ProcessState {
        // Safety: The data in the const_data section of the process structure
        // is constant and thus allowed to be read atomically at any time.
        // Furthermore, the pointer within the unsafe cell should not alias,
        // and will only be dropped when there are no more references to it.
        unsafe { self.inner.get().as_mut().unwrap() }
            .atomic_data
            .status
            .load(core::sync::atomic::Ordering::AcqRel)
    }
}

// Safety:
// A process interface is allowed to be shared between threads, this is because
// it is only a handle to the actual process, which will have access to the
// dangerous parts mediated by a mutex
unsafe impl Send for ProcessInterface {}
unsafe impl Sync for ProcessInterface {}
