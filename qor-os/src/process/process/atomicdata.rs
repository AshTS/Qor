use atomic::Atomic;

use super::*;

/// Atomic Process Data
pub struct AtomicProcessData {
    pub state: Atomic<ProcessState>,
}

impl AtomicProcessData {
    pub fn new() -> Self {
        Self {
            state: Atomic::new(ProcessState::Running)
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
        self.state.store(state, core::sync::atomic::Ordering::SeqCst)
    }
}