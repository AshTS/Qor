use super::*;

/// Inner process structure
pub struct Process {
    pub atomic_data: AtomicProcessData,
    pub const_data: ConstantProcessData,
    pub mutable_data: MutableProcessData,
}
