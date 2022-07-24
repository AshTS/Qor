//! Memory management for the Qor Kernel

/// Size of a page of memory
pub const PAGE_SIZE: usize = 4096;

/// Type to conveniently refer to a page of memory
pub type Page = [u8; PAGE_SIZE];

/// Global Page Allocator
pub static PAGE_ALLOCATOR: GlobalPageAllocator = GlobalPageAllocator::new();

pub mod error;
pub use error::*;

pub mod pages;
pub use pages::*;

pub mod pagetable;
pub use pagetable::*;