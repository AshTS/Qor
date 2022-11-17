use libutils::sync::{semaphore::SignalSemaphore};

use crate::mem::{KernelPageBox, PageTable};

use super::ExecutionState;

/// Mutable Process Data
pub struct MutableProcessData {
    execution_state: ExecutionState,
    memory_map: KernelPageBox<PageTable>,
    wait_semaphore: Option<SignalSemaphore>,
}

impl MutableProcessData {
    /// Construct a new mutable process data component from its constituent components
    pub fn new(execution_state: ExecutionState, memory_map: KernelPageBox<PageTable>) -> Self {
        Self {
            execution_state,
            memory_map,
            wait_semaphore: None,
        }
    }

    /// Get the data for the process switch into this process
    pub fn get_proc_switch_data(&self) -> (usize, usize, usize) {
        self.execution_state.switch_to(self.memory_map.raw())
    }
}
