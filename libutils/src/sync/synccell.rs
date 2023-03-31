use core::cell::UnsafeCell;

use super::Mutex;
use super::MutexGuard;

/// A synchronization primitive similar to the `Mutex`, but allows simultaneous
/// shared reads, with locking to allow unique references to be created.
/// Because of the presence of the shared access, atomic reference counting is
/// used to keep track of the shared references. There are two forms of guards
/// on the `SyncCell` a `UniqueGuard` and a `SharedGuard`.
pub struct SyncCell<T> {
    inner: UnsafeCell<T>,
    lock: Mutex<()>,
    reference_count: core::sync::atomic::AtomicUsize,
    strong_wait: core::sync::atomic::AtomicUsize,
}

impl<T> SyncCell<T> {
    /// Create a new `SyncCell` around an inner object
    pub const fn new(inner: T) -> Self {
        Self {
            inner: UnsafeCell::new(inner),
            lock: Mutex::new(()),
            reference_count: core::sync::atomic::AtomicUsize::new(0),
            strong_wait: core::sync::atomic::AtomicUsize::new(0)
        }
    }

    /// Attempt to get a `SharedGuard`
    pub fn attempt_shared(&self) -> Option<SharedGuard<T>> {
        if let Some(lock) = self.lock.attempt_lock() {

            if self.strong_wait.load(core::sync::atomic::Ordering::SeqCst) != 0 {
                return None;
            }

            self.reference_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst);

            let guard = SharedGuard { reference: self };

            drop(lock);

            Some(guard)
        }
        else {
            None
        }
    }

    /// Attempt to get the lock on the wrapped `Mutex`, returning `None` if it is not possible
    fn attempt_unique_internal(&self) -> Option<UniqueGuard<T>> {
        self.lock.attempt_lock().map(|l| {
            if self.reference_count.load(core::sync::atomic::Ordering::SeqCst) == 0 {
                self.pop_strong_wait();
                Some(UniqueGuard { reference: self, _inner_guard: l })
            }
            else {
                None
            }
        })?
    }

    /// Attempt to get the lock on the wrapped `Mutex`, returning `None` if it is not possible
    pub fn attempt_unique(&self) -> Option<UniqueGuard<T>> {
        self.push_strong_wait();

        if let Some(v) = self.attempt_unique_internal() {
            Some(v)
        }
        else {
            self.pop_strong_wait();
            None
        }
    }

    /// Push a strong waiting signal
    fn push_strong_wait(&self) {
        self.strong_wait.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
    }

    /// Pop a strong waiting signal
    fn pop_strong_wait(&self) {
        self.strong_wait.fetch_sub(1, core::sync::atomic::Ordering::SeqCst);
    }

    /// Asynchronously request shared access
    pub fn async_shared(&self) -> SyncCellSharedFuture<T> {
        SyncCellSharedFuture { synccell: self }
    }

    /// Asynchronously request unique access
    pub fn async_unique(&self) -> SyncCellUniqueFuture<T> {
        self.push_strong_wait();
        SyncCellUniqueFuture { synccell: self }
    }

    /// Spin until shared access can be acquired
    pub fn spin_shared(&self) -> SharedGuard<T> {
        loop {
            if let Some(guard) = self.attempt_shared() {
                return guard;
            }
        }
    }

    /// Spin until unique access can be acquired
    pub fn spin_unique(&self) -> UniqueGuard<T> {
        self.push_strong_wait();
        loop {
            if let Some(guard) = self.attempt_unique_internal() {
                return guard;
            }
        }
    }
}

unsafe impl<T> Send for SyncCell<T> {}
unsafe impl<T> Sync for SyncCell<T> {}

/// `UniqueGuard` object which gives access to the wrapped object, can only be
/// constructed from the `SyncCell` which owns the wrapped data. Allows unique
/// access to the wrapped value
pub struct UniqueGuard<'a, T> {
    reference: &'a SyncCell<T>,
    _inner_guard: MutexGuard<'a, ()>
}

impl<'a, T> core::ops::Deref for UniqueGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // Safety: This is safe because the only way to acquire a `UniqueGuard` is to lock the internal `Mutex` within the `SyncCell`
        unsafe { self.reference.inner.get().as_ref().unwrap() }
    }
}

impl<'a, T> core::ops::DerefMut for UniqueGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Safety: This is safe because the only way to acquire a `UniqueGuard` is to lock the internal `Mutex` within the `SyncCell`
        unsafe { self.reference.inner.get().as_mut().unwrap() }
    }
}

/// `SharedGuard` object which gives access to the wrapped object, can only be
/// constructed from the `SyncCell` which owns the wrapped data. Allows shared
/// access to the wrapped value
pub struct SharedGuard<'a, T> {
    reference: &'a SyncCell<T>,
}

impl<'a, T> core::ops::Deref for SharedGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // Safety: This is safe because a `SharedGuard` cannot be created while the mutex inside the `SyncCell` is locked, thus preventing a `UniqueGuard` from being given out
        unsafe { self.reference.inner.get().as_ref().unwrap() }
    }
}

impl<'a, T> core::ops::Drop for SharedGuard<'a, T> {
    fn drop(&mut self) {
        self.reference.reference_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst);
    }
}

/// A future implementor for the `SyncCell` which allows async access to the `UniqueGuard`
pub struct SyncCellUniqueFuture<'a, T> {
    synccell: &'a SyncCell<T>,
}

impl<'a, T> core::future::Future for SyncCellUniqueFuture<'a, T> {
    type Output = UniqueGuard<'a, T>;

    fn poll(self: core::pin::Pin<&mut Self>, _: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        match self.synccell.attempt_unique_internal() {
            Some(lock) => core::task::Poll::Ready(lock),
            None => core::task::Poll::Pending
        }
    }
}

/// A future implementor for the `SyncCell` which allows async access to the `UniqueGuard`
pub struct SyncCellSharedFuture<'a, T> {
    synccell: &'a SyncCell<T>,
}

impl<'a, T> core::future::Future for SyncCellSharedFuture<'a, T> {
    type Output = SharedGuard<'a, T>;

    fn poll(self: core::pin::Pin<&mut Self>, _: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        match self.synccell.attempt_shared() {
            Some(lock) => core::task::Poll::Ready(lock),
            None => core::task::Poll::Pending
        }
    }
}