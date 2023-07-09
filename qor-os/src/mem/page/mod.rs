pub mod bump;
pub use bump::*;

/// Size of a page in bytes
pub const PAGE_SIZE: usize = 4096;

/// A page of memory
#[repr(C)]
#[repr(align(4096))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Page(pub [u8; PAGE_SIZE]);

impl core::default::Default for Page {
    fn default() -> Self {
        Self([0; PAGE_SIZE])
    }
}

/// Fallback, empty region of memory for statically initializing the bump allocator
static FALLBACK_REGION: [Page; 0] = [];

/// Page-scale static bump allocator
pub static BUMP_ALLOC: KernelPageStaticBumpAllocator = unsafe { KernelPageStaticBumpAllocator::new(FALLBACK_REGION.as_ptr_range()) };