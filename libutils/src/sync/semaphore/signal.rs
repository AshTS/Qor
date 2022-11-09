use super::Semaphore;

/// The read end of a signal semaphore
pub struct SignalSemaphore {
    flag: alloc::sync::Arc<core::sync::atomic::AtomicBool>
}

/// The write end of a signal semaphore
pub struct SignalSemaphoreSender {
    flag: alloc::sync::Arc<core::sync::atomic::AtomicBool>
}

impl Semaphore for SignalSemaphore {
    fn read(self) -> (bool, Option<Self>) {
        (self.flag.swap(false, core::sync::atomic::Ordering::SeqCst), Some(self))
    }

    unsafe fn unchecked_read(&mut self) -> bool {
        self.flag.swap(false, core::sync::atomic::Ordering::SeqCst)
    }
}

impl SignalSemaphoreSender {
    /// Send a signal to the reading end of the Semaphore, returns false if two signals have collided (the reader hasn't cleared the flag yet)
    fn send(&self) -> bool {
        !self.flag.swap(true, core::sync::atomic::Ordering::SeqCst)
    }
}

/// Construct a signal semaphore pair
pub fn signal_semaphor_pair() -> (SignalSemaphore, SignalSemaphoreSender) {
    let flag = alloc::sync::Arc::new(core::sync::atomic::AtomicBool::new(false));

    (SignalSemaphore { flag: flag.clone() },
     SignalSemaphoreSender { flag })
}