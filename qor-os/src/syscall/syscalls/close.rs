use crate::*;

pub fn syscall_close(process: &mut process::process::ProcessData, fd: usize) -> usize
{
    process.data.close_fd(unsafe { fs::INTERFACE.as_mut().unwrap() }, fd)
}