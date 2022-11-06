use atomic::Atomic;

use super::*;

/// Atomic Process Data
pub struct AtomicProcessData {
    pub status: Atomic<ProcessState>,
}
