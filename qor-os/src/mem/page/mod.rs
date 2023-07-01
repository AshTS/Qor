pub mod bump;
pub use bump::*;

/// Size of a page in bytes
pub const PAGE_SIZE: usize = 4096;

/// A page of memory
pub type Page = [u8; PAGE_SIZE];