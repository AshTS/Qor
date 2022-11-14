pub mod executor;
pub use executor::*;

pub mod task;
pub use task::*;

/// Spawn a new executor to run a task in full syncronously
pub fn execute_task(task: impl core::future::Future<Output = ()> + 'static) {
    let mut executor = SimpleExecutor::new();
    executor.spawn(Task::new(task));
    executor.run();
}