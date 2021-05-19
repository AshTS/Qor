pub mod manager;
pub mod info;
pub mod init;
pub mod process;
pub mod scheduler;

use crate::*;

use self::manager::ProcessManager;

static PROCESS_MANAGER: core::sync::atomic::AtomicPtr<ProcessManager> = core::sync::atomic::AtomicPtr::new(0 as *mut ProcessManager);

pub use scheduler::process_switch;

/// Initialize the process manager
pub fn init_process_manager()
{
    let manager = Box::new(ProcessManager::new());
    let reference = Box::leak(manager);


    PROCESS_MANAGER.store(reference as *mut ProcessManager, core::sync::atomic::Ordering::SeqCst);
    
    kprintln!("Initialized Process Manager");
}

/// Get the process manager
pub fn get_process_manager() -> Option<&'static mut ProcessManager>
{
    if PROCESS_MANAGER.load(core::sync::atomic::Ordering::SeqCst).is_null()
    {
        return None;
    }

    // Safety: The only way to instantiate this is by calling `init_process_manager`, and this will fail if it hasn't been initialized 
    unsafe { PROCESS_MANAGER.load(core::sync::atomic::Ordering::SeqCst).as_mut() }
}