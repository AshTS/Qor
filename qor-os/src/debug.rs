/// Categories for debug prints
#[derive(PartialEq, Eq)]
pub enum DebugCategories
{
    BlockDevice,
    ByteMemoryAllocation,
    Elf,
    Filesystem,
    Initialization,
    Interrupts,
    KernelPageTable,
    MemoryAllocation,
    MemoryMapping,
    Processes,
    Scheduling,
    Syscalls,
    VirtIO,
    Other,
}

// Flags for debug prints
pub const ALL: bool = true;

#[cfg(not(test))]
pub const BLOCK_DEVICE: bool = false;
#[cfg(test)]
pub const BLOCK_DEVICE: bool = false;

#[cfg(not(test))]
pub const BYTE_MEMORY_ALLOCATION: bool = false;
#[cfg(test)]
pub const BYTE_MEMORY_ALLOCATION: bool = false;

#[cfg(not(test))]
pub const ELF: bool = false;
#[cfg(test)]
pub const ELF: bool = false;

#[cfg(not(test))]
pub const FILESYSTEM: bool = false;
#[cfg(test)]
pub const FILESYSTEM: bool = false;

#[cfg(not(test))]
pub const INITIALIZATION: bool = true;
#[cfg(test)]
pub const INITIALIZATION: bool = true;

#[cfg(not(test))]
pub const INTERRUPTS: bool = false;
#[cfg(test)]
pub const INTERRUPTS: bool = false;

#[cfg(not(test))]
pub const KERNEL_PAGE_TABLE: bool = false;
#[cfg(test)]
pub const KERNEL_PAGE_TABLE: bool = false;

#[cfg(not(test))]
pub const MEMORY_ALLOCATION: bool = false;
#[cfg(test)]
pub const MEMORY_ALLOCATION: bool = false;

#[cfg(not(test))]
pub const MEMORY_MAPPING: bool = false;
#[cfg(test)]
pub const MEMORY_MAPPING: bool = false;

#[cfg(not(test))]
pub const PROCESSES: bool = true;
#[cfg(test)]
pub const PROCESSES: bool = false;

#[cfg(not(test))]
pub const SCHEDULING: bool = false;
#[cfg(test)]
pub const SCHEDULING: bool = false;

#[cfg(not(test))]
pub const SYSCALLS: bool = false;
#[cfg(test)]
pub const SYSCALLS: bool = false;

#[cfg(not(test))]
pub const VIRTIO: bool = false;
#[cfg(test)]
pub const VIRTIO: bool = false;

/// Helper function to determine if a debug print should occur
pub const fn check_debug(cat: DebugCategories) -> bool
{
    if !ALL
    {
        false
    }
    else
    {
        match cat
        {
            DebugCategories::BlockDevice => BLOCK_DEVICE,
            DebugCategories::ByteMemoryAllocation => BYTE_MEMORY_ALLOCATION,
            DebugCategories::Elf => ELF,
            DebugCategories::Filesystem => FILESYSTEM,
            DebugCategories::Initialization => INITIALIZATION,
            DebugCategories::Interrupts => INTERRUPTS,
            DebugCategories::KernelPageTable => KERNEL_PAGE_TABLE,
            DebugCategories::MemoryAllocation => MEMORY_ALLOCATION,
            DebugCategories::MemoryMapping => MEMORY_MAPPING,
            DebugCategories::Processes => PROCESSES,
            DebugCategories::Scheduling => SCHEDULING,
            DebugCategories::Syscalls => SYSCALLS,
            DebugCategories::VirtIO => VIRTIO,
            DebugCategories::Other => true // This defaults to true to allow unspecified prints to pass
        }
    }
}