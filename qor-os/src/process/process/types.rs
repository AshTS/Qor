/// Process ID Datatype
pub type ProcessIdentifier = u64;

/// Reasons for a process to wait
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitReason {
    ForChildren,
    Semaphore,
}

/// Process State
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Pending,
    Running,
    Zombie,
    Waiting(WaitReason),
    Dead,
}

impl core::default::Default for ProcessState {
    fn default() -> Self {
        ProcessState::Pending
    }
}
