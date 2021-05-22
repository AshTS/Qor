/// Categories for debug prints
#[derive(PartialEq, Eq)]
pub enum DebugCategories
{
    Other,
    Initialization,
    KernelPageTable,
}

// Flags for debug prints
pub const ALL: bool = true;
pub const INITIALIZATION: bool = true;
pub const KERNEL_PAGE_TABLE: bool = true;

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
            DebugCategories::Initialization => INITIALIZATION,
            DebugCategories::KernelPageTable => KERNEL_PAGE_TABLE,
            DebugCategories::Other => true // This defaults to true to allow unspecified prints to pass
        }
    }
}