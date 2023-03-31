pub mod executor;
pub use executor::*;

pub mod task;
pub use task::*;

/// Global Executor
static mut GLOBAL_EXECUTOR: Option<alloc::sync::Arc<libutils::sync::Mutex<SimpleExecutor>>> = None;

/// Initialize the global executor
pub fn init_global_executor(_marker: libutils::sync::InitThreadMarker) {
    // Safety: Becuase we hold the init thread marker, this access cannot alias
    let _ = unsafe { &mut GLOBAL_EXECUTOR }.insert(alloc::sync::Arc::new(
        libutils::sync::Mutex::new(SimpleExecutor::new()),
    ));
}

/// Add a task to the global executor
pub fn add_global_executor_task(task: impl core::future::Future<Output = ()> + 'static) {
    // Safety: We do not require mutable access, and thus we don't alias when accessing
    unsafe { (GLOBAL_EXECUTOR.as_ref()).expect("Global Executor Not Initialized") }
        .spin_lock()
        .spawn(Task::new(task));
}

/// Run all executor tasks until all are pending or completed
pub fn run_global_executor_step() -> bool {
    // Safety: We do not require mutable access, and thus we don't alias when accessing
    unsafe { (GLOBAL_EXECUTOR.as_ref()).expect("Global Executor Not Initialized") }
        .spin_lock()
        .run_until_pending()
}

/// Spawn a new executor to run a task in full syncronously
pub fn execute_task(task: impl core::future::Future<Output = ()> + 'static) {
    let mut executor = SimpleExecutor::new();
    executor.spawn(Task::new(task));
    executor.run();
}
