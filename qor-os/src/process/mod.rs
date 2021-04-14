mod manager;
mod process;
mod scheduler;

use crate::*;

use self::manager::ProcessManager;

static PROCESS_MANAGER: core::sync::atomic::AtomicPtr<ProcessManager> = core::sync::atomic::AtomicPtr::new(0 as *mut ProcessManager);

pub use scheduler::schedule_next;

/// Initialize the process manager
pub fn init_process_manager()
{
    let manager = Box::new(ProcessManager::new());
    let reference = Box::leak(manager);


    PROCESS_MANAGER.store(reference as *mut ProcessManager, core::sync::atomic::Ordering::SeqCst);
    
    kprintln!("Initialized Process Manager");
}

/// Get the process manager
pub fn get_process_manager() -> &'static mut ProcessManager
{
    // Safety: The only way to instantiate this is by calling `init_process_manager`, and this will fail if it hasn't been initialized 
    unsafe { PROCESS_MANAGER.load(core::sync::atomic::Ordering::SeqCst).as_mut().unwrap() }
}