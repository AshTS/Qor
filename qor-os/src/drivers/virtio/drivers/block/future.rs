use super::*;
use libutils::sync::semaphore::Semaphore;
use core::future::Future;
use core::pin::Pin;
use core::task::Poll;
use core::task::Context;

/// Future for a block device write
pub struct AsyncBlockWrite {
    pub operation: Option<BlockOperation>
}

/// Future for a block device read
pub struct AsyncBlockRead {
    pub operation: Option<BlockOperation>
}

impl Future for AsyncBlockWrite {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
        if let Some(operation) = &mut self.operation {
            // Safety: Because we enforce the drop of this operation via an option, we safely destroy the value
            let done = unsafe { operation.unchecked_read() };

            if done {
                self.operation = None;
                Poll::Ready(())
            }
            else {
                Poll::Pending
            }
        }
        else {
            unreachable!()
        }
    }
}

impl Future for AsyncBlockRead {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
        if let Some(operation) = &mut self.operation {
            // Safety: Because we enforce the drop of this operation via an option, we safely destroy the value
            let done = unsafe { operation.unchecked_read() };

            if done {
                self.operation = None;
                Poll::Ready(())
            }
            else {
                Poll::Pending
            }
        }
        else {
            unreachable!()
        }
    }
}