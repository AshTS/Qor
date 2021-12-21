//! The dev filesystem is to be mounted at /dev/ and gives access to various
//! devices


pub mod fs;
pub use fs::*;

mod devices;
pub mod tty;

mod tty_consts;