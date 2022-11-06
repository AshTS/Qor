/// Process ID Datatype
pub type ProcessIdentifier = u64;

/// Reasons for a process to wait
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitReason {
    ForChildren,
}

/// Process State
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Running,
    Zombie,
    Waiting(WaitReason),
    Dead,
}
