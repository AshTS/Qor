pub mod process;
use libutils::sync::{InitThreadMarker, SyncCell};
pub use process::*;

pub mod scheduler;
pub use scheduler::*;

use alloc::collections::BTreeMap;
use alloc::sync::Arc;

use atomic::Atomic;

static mut PROCESS_MAP: Option<Arc<SyncCell<BTreeMap<ProcessIdentifier, ProcessInterface>>>> = None;

static NEXT_PID: Atomic<ProcessIdentifier> = Atomic::new(0);

/// Initialize the process map
pub fn init_process_map(_marker: InitThreadMarker) {
    // Safety: We have the single thread marker, so this reference will never alias
    unsafe { PROCESS_MAP.replace(Arc::new(SyncCell::new(BTreeMap::new()))) };
}

/// Get a reference to the global process map
pub fn process_map() -> Arc<SyncCell<BTreeMap<ProcessIdentifier, ProcessInterface>>> {
    // Safety: This value can never be updated after the initial initialization, thus it is safe to get a shared reference to it.
    if let Some(proc_map) = unsafe { &PROCESS_MAP } {
        proc_map.clone()
    } else {
        panic!("process map not yet initialized");
    }
}

/// Get the next ProcessIdentifier
pub fn next_process_id() -> ProcessIdentifier {
    NEXT_PID.fetch_add(1, core::sync::atomic::Ordering::SeqCst)
}

/// Add a process to the process map
pub fn add_process(process: Process) {
    let pid = process.pid();

    kdebugln!(unsafe "Adding Process {}", pid);

    process_map()
        .spin_unique()
        .insert(pid, ProcessInterface::new(alloc::sync::Arc::new(process)));
}

/// Get the process interface for the given pid
pub fn get_process(pid: ProcessIdentifier) -> Option<ProcessInterface> {
    process_map().spin_shared().get(&pid).map(|v| v.clone())
}
