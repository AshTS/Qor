use super::Semaphore;

/// Watches a particular memory address, signaling when it has changed
pub struct UpdateSemaphore<T: Copy + PartialEq> {
    ptr: *mut T,
    last: T
}

impl<T: Copy + PartialEq> UpdateSemaphore<T> {
    /// Construct a new update semaphore
    pub unsafe fn new(ptr: *mut T, original: T) -> Self {
        Self {
            ptr,
            last: original
        }
    }

    // Construct a new update semaphore and read the initial value
    pub unsafe fn new_read(ptr: *mut T) -> Self {
        Self {
            ptr,
            last: core::ptr::read_volatile(ptr)
        }
    }
}

impl<T: Copy + PartialEq> Semaphore for UpdateSemaphore<T> {
    fn read(self) -> (bool, Option<Self>) {
        if unsafe { core::ptr::read_volatile(self.ptr) } != self.last {
            (true, None)
        }
        else {
            (false, Some(self))
        }
    }

    unsafe fn unchecked_read(&mut self) -> bool {
        let new = unsafe { core::ptr::read_volatile(self.ptr) };
        if new != self.last {
            self.last = new;
            true
        }
        else {
            false
        }
    }
}

// Safety:  This is considered safe only because we consume the semaphore when it detects an update, this way, we don't have to worry about partial updates causing a double trigger
unsafe impl<T: Copy + PartialEq> Send for UpdateSemaphore<T> {}
unsafe impl<T: Copy + PartialEq> Sync for UpdateSemaphore<T> {}