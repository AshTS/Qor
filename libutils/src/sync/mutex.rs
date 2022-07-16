use core::cell::UnsafeCell;

/// A `Mutex` implementation for the Qor kernel, simply wraps an `UnsafeCell`
/// with an `AtomicBool` used as a flag to denote if the `Mutex` is locked or
/// not. The wrapped object can be accessed by `lock`ing the `Mutex` which
/// returns a `MutexGuard` which allows access to the inner object, unlocking
/// when it is dropped.
pub struct Mutex<T>
{
    inner: UnsafeCell<T>,
    is_locked: core::sync::atomic::AtomicBool
}

impl<T> Mutex<T>
{
    /// Create a new `Mutex` around an inner object
    pub const fn new(inner: T) -> Self
    {
        Self
        {
            inner: UnsafeCell::new(inner),
            is_locked: core::sync::atomic::AtomicBool::new(false)
        }
    }

    /// Spin until the lock can be acquired, returning a `MutexGuard` for the wrapped data
    pub fn spin_lock<'a>(&'a self) -> MutexGuard<'a, T>
    {
        while !self.acquire_lock() {}

        MutexGuard { reference: &self }
    }

    /// Internal function to attempt to acquire the lock on the `Mutex`,
    /// returns `true` if the lock was acquired. 
    fn acquire_lock(&self) -> bool
    {
        // We return the inverse, because we only want to say we acquired the lock if the transition was from false to true.
        !self.is_locked.swap(true, core::sync::atomic::Ordering::Acquire)
    }

    /// Internal function to release the lock, this is marked as unsafe as if it is called unnecessarily, it will allow multiple mutable references
    unsafe fn release_lock(&self)
    {
        self.is_locked.store(false, core::sync::atomic::Ordering::Release);
    }
}

unsafe impl<T> Send for Mutex<T> {}
unsafe impl<T> Sync for Mutex<T> {}

/// `MutexGuard` object which gives access to the wrapped object, can only be
/// constructed from the `Mutex` which owns the wrapped data.
pub struct MutexGuard<'a, T>
{
    reference: &'a Mutex<T>
}

impl<'a, T> core::ops::Deref for MutexGuard<'a, T>
{
    type Target = T;

    fn deref(&self) -> &Self::Target
    {
        // Safety: This is safe because the only way to acquire a `MutexGuard` is to lock a `Mutex`
        unsafe
        {
            self.reference.inner.get().as_ref().unwrap()
        }
    }
}

impl<'a, T> core::ops::DerefMut for MutexGuard<'a, T>
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        // Safety: This is safe because the only way to acquire a `MutexGuard` is to lock a `Mutex`
        unsafe
        {
            self.reference.inner.get().as_mut().unwrap()
        }
    }
}

impl<'a, T> core::ops::Drop for MutexGuard<'a, T>
{
    fn drop(&mut self)
    {
        unsafe { self.reference.release_lock() }
    }
}