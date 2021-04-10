pub mod addrs;
pub mod heap;
pub mod kernel;
pub mod mmu;
pub mod pages;
pub mod pagetable;

pub use heap::{kalloc, kzalloc, kfree};
pub use mmu::{kvalloc, kvfree};

pub use pagetable::EntryBits;