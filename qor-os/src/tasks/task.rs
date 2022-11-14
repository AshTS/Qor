use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
use core::task::Context;
use core::task::Poll;

/// Kernel task object used to enable async execution on the kernel
pub struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    /// Construct a task around a properly designed future
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            future: Box::pin(future),
        }
    }

    /// Poll the wrapped future
    pub fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}
