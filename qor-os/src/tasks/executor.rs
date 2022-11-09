use super::Task;
use alloc::collections::VecDeque;
use core::task::Waker;
use core::task::RawWaker;
use core::task::RawWakerVTable;
use core::task::Context;
use core::task::Poll;

/// Kernel executor
pub struct SimpleExecutor {
    task_queue: VecDeque<Task>
}

impl SimpleExecutor {
    /// Construct a new empty executor
    pub fn new() -> SimpleExecutor {
        SimpleExecutor {
            task_queue: VecDeque::new()
        }
    }

    /// Add a new task to the spawn
    pub fn spawn(&mut self, task: Task) {
        self.task_queue.push_back(task);
    }

    /// Single step, returns true when there is at least one task in the queue
    pub fn step(&mut self) -> Option<bool> {
        if let Some(mut task) = self.task_queue.pop_front() {
            let waker = dummy_waker();
            let mut context = Context::from_waker(&waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => Some(true),
                Poll::Pending => { self.task_queue.push_back(task); Some(false) }
            }            
        }
        else {
            None
        }
    }

    /// Run to exhaustion
    pub fn run(&mut self) {
        while self.step().is_some() {}
    }

    /// Run through the queue until all tasks are pending
    pub fn run_until_pending(&mut self) {
        'outer: loop {
            let remaining = self.task_queue.len();

            let mut flag = false;

            for _ in 0..remaining {
                if let Some(b) = self.step() {
                    flag |= b;
                }
                else {
                    break 'outer;
                }
            }

            if !flag {
                break;
            }
        }
    }
}

fn dummy_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(0 as *const (), vtable)
}

fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}