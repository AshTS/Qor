use crate::*;

pub fn syscall_exit(process: &mut process::process::ProcessData, code: usize)
{
    kdebugln!(Syscall, "Exiting PID {} With Code `{}`", process.get_pid(), code);

    process.halt();
}