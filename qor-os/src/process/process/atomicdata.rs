use atomic::Atomic;
use libutils::sync::{semaphore::{SignalSemaphore, SignalSemaphoreSender}, SyncCell};

use super::*;

/// Atomic Process Data
pub struct AtomicProcessData {
    state: Atomic<ProcessState>,
    child_semaphore: SignalSemaphore,
    pub child_semaphore_send: SignalSemaphoreSender,
    waiting_semaphore: SyncCell<Option<SignalSemaphore>>
}

impl AtomicProcessData {
    pub fn new() -> Self {
        let (read, write) = libutils::sync::semaphore::signal_semaphor_pair();

        Self {
            state: Atomic::new(ProcessState::Pending),
            child_semaphore: read,
            child_semaphore_send: write,
            waiting_semaphore: SyncCell::new(None)
        }
    }
}

// Getters and setters
impl AtomicProcessData {
    /// Get the current process state
    pub fn state(&self) -> ProcessState {
        self.state.load(core::sync::atomic::Ordering::SeqCst)
    }

    /// Set the current process state
    pub fn set_state(&self, state: ProcessState) {
        self.state
            .store(state, core::sync::atomic::Ordering::SeqCst)
    }

    /// Check the child pending semaphore
    pub fn check_child_semaphore(&self) -> bool {
        self.child_semaphore.read_atomic()
    }

    /// Check the waiting semaphore
    pub fn check_wait_semaphore(&self) -> Option<bool> {
        self.waiting_semaphore.attempt_shared().map(|g| g.as_ref().map(|s| s.read_atomic()))?
    }

    /// Get a new sender for the wait semaphore
    pub fn new_wait_semaphore(&self) -> SignalSemaphoreSender {
        let (read, write) = libutils::sync::semaphore::signal_semaphor_pair();
        let _ = self.waiting_semaphore.spin_unique().insert(read);
        write
    }
}
