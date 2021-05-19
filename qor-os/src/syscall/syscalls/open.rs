use crate::*;

pub fn syscall_open(process: &mut process::process::ProcessData, ptr: usize, mode: usize) -> usize
{
    let name = unsafe { super::to_str(process.map_ptr(ptr)) };

    kdebugln!(FileSystemSyscall, "Attempting to open file `{}` with mode 0x{:x}", name, mode);

    process.data.open_fd(unsafe { fs::INTERFACE.as_mut().unwrap() }, name, mode)
}