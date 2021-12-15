use crate::*;

type Semaphore = alloc::sync::Arc<core::sync::atomic::AtomicBool>;

pub struct BlockingSemaphore
{
    semaphore: Semaphore,
    reason: String,
}

impl BlockingSemaphore
{
    pub fn new(reason: String) -> (Self, alloc::sync::Arc<core::sync::atomic::AtomicBool>)
    {
        let semaphore = alloc::sync::Arc::new(core::sync::atomic::AtomicBool::new(false));

        let blocking = Self { semaphore: semaphore.clone(), reason };

        (blocking, semaphore)
    }

    pub fn from_current(reason: String, current: Semaphore) -> Self
    {
        Self
        {
            semaphore: current.clone(),
            reason
        }
    }

    pub fn check(self) -> bool
    {
        self.semaphore.load(core::sync::atomic::Ordering::AcqRel)
    }
}