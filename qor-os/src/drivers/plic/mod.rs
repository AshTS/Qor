pub mod mmio;
pub use mmio::*;

pub mod priorities;
pub use priorities::*;

pub mod raw;

pub type InterruptID = u32;
