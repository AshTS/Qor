// Modules
pub mod data;
pub mod descriptor;
pub mod elf;
pub mod init;
pub mod process;
pub mod scheduler;
pub mod stats;
pub mod signals;

mod pipe;

pub type PID = u16;