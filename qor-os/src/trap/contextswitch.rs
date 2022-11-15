/// Function called when a context switch must occur
pub fn context_switch() -> ! {
    loop {
        crate::tasks::run_global_executor_step();
        crate::process::schedule();
    }
}