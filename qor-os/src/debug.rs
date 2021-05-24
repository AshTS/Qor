/// Categories for debug prints
#[derive(PartialEq, Eq)]
pub enum DebugCategories
{
    ByteMemoryAllocation,
    Initialization,
    KernelPageTable,
    MemoryAllocation,
    Other,
}

// Flags for debug prints
pub const ALL: bool = true;
pub const BYTE_MEMORY_ALLOCATION: bool = false;
pub const INITIALIZATION: bool = true;
pub const KERNEL_PAGE_TABLE: bool = false;
pub const MEMORY_ALLOCATION: bool = false;

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
            DebugCategories::KernelPageTable => KERNEL_PAGE_TABLE,
            DebugCategories::MemoryAllocation => MEMORY_ALLOCATION,
            DebugCategories::Other => true // This defaults to true to allow unspecified prints to pass
        }
    }
}