use super::*;

/// Constant Process Data
pub struct ConstantProcessData {
    pub pid: ProcessIdentifier,
}

impl ConstantProcessData {
    /// Allocate a new process with constant data
    pub fn new(pid: ProcessIdentifier) -> Self {
        Self {
            pid
        }
    }

    /// Get the Process Identifier of a process
    pub fn pid(&self) -> ProcessIdentifier {
        self.pid
    }
}