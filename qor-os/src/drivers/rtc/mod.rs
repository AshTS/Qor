//! Real Time Clock Driver

// TODO: This is a hack and should be done in userspace later
pub const LOCALIZATION_OFFSET: i64 = -5 * 3600;

pub mod driver;
pub use driver::*;

pub mod structs;
pub use structs::*;