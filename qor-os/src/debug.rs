/// Categories for debug prints
#[derive(PartialEq, Eq)]
pub enum DebugCategories
{
    ByteMemoryAllocation,
    Initialization,
    Interrupts,
    KernelPageTable,
    MemoryAllocation,
    MemoryMapping,
    Other,
}

// Flags for debug prints
pub const ALL: bool = true;

#[cfg(not(test))]
pub const BYTE_MEMORY_ALLOCATION: bool = false;
#[cfg(test)]
pub const BYTE_MEMORY_ALLOCATION: bool = false;

#[cfg(not(test))]
pub const INITIALIZATION: bool = true;
#[cfg(test)]
pub const INITIALIZATION: bool = true;

#[cfg(not(test))]
pub const INTERRUPTS: bool = true;
#[cfg(test)]
pub const INTERRUPTS: bool = true;

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
            DebugCategories::ByteMemoryAllocation => BYTE_MEMORY_ALLOCATION,
            DebugCategories::Initialization => INITIALIZATION,
            DebugCategories::Interrupts => INTERRUPTS,
            DebugCategories::KernelPageTable => KERNEL_PAGE_TABLE,
            DebugCategories::MemoryAllocation => MEMORY_ALLOCATION,
            DebugCategories::MemoryMapping => MEMORY_MAPPING,
            DebugCategories::Other => true // This defaults to true to allow unspecified prints to pass
        }
    }
}