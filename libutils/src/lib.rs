//! Utilities Library for the Qor Kernel
#![no_std]

#[cfg(feature = "std")]
extern crate std;

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod paths;
pub mod sync;
pub mod utils;