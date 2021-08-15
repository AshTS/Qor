use crate::*;

/// sync Syscall
pub fn syscall_sync(proc: &mut super::Process) -> usize
{
    kdebugln!(Syscalls, "PID {} requests fs sync", proc.pid);

    use fs::fstrait::Filesystem;
    fs::vfs::get_vfs_reference().unwrap().sync().unwrap();

    0
}