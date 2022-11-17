use libutils::sync::{semaphore::{SignalSemaphore, SignalSemaphoreSender}, SyncCell};

/// Atomic Process Data
pub struct AtomicProcessData {
    child_semaphore: SignalSemaphore,
    pub child_semaphore_send: SignalSemaphoreSender,
    waiting_semaphore: SyncCell<Option<SignalSemaphore>>
}

impl AtomicProcessData {
    pub fn new() -> Self {
        let (read, write) = libutils::sync::semaphore::signal_semaphor_pair();

        Self {
            child_semaphore: read,
            child_semaphore_send: write,
            waiting_semaphore: SyncCell::new(None)
        }
    }
}

// Getters and setters
impl AtomicProcessData {
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
