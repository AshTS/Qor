use crate::mem::PAGE_SIZE;

/// Global Page Allocator Error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobalPageAllocatorError {
    NotInitialized,
    OutOfMemory,
    BitmapSpaceError,
    MemoryNotAllocated(*mut [u8; PAGE_SIZE], usize),
}

impl core::fmt::Display for GlobalPageAllocatorError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            GlobalPageAllocatorError::NotInitialized => write!(f, "Allocator Not Initialized"),
            GlobalPageAllocatorError::OutOfMemory => write!(f, "Out of Memory"),
            GlobalPageAllocatorError::BitmapSpaceError => write!(f, "Not Enough Space For Bitmap"),
            GlobalPageAllocatorError::MemoryNotAllocated(ptr, page_count) => write!(
                f,
                "{} Page{} at {:?} Not Allocated",
                page_count,
                if *page_count > 1 { "s" } else { "" },
                ptr
            ),
        }
    }
}
