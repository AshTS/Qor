use crate::*;

pub fn syscall_exit(process: &process::process::ProcessData, code: usize)
{
    kprintln!("Exiting PID {} With Code `{}`", process.get_pid(), code);
}