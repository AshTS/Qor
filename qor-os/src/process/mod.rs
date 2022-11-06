pub mod process;
use libutils::sync::{InitThreadMarker, Mutex};
pub use process::*;

use alloc::collections::BTreeMap;
use alloc::sync::Arc;

static mut PROCESS_MAP: Option<Arc<Mutex<BTreeMap<ProcessIdentifier, Arc<ProcessInterface>>>>> =
    None;

/// Initialize the process map
pub fn init_process_map(_marker: InitThreadMarker) {
    unsafe { PROCESS_MAP.replace(Arc::new(Mutex::new(BTreeMap::new()))) };
}

/// Get a reference to the global process map
pub fn process_map() -> Arc<Mutex<BTreeMap<ProcessIdentifier, Arc<ProcessInterface>>>> {
    if let Some(proc_map) = unsafe { &PROCESS_MAP } {
        proc_map.clone()
    } else {
        panic!("process map not yet initialized");
    }
}
