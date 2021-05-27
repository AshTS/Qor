//! Syscall Handling

use crate::*;

use process::process::Process;

// Modules
mod exit;

/// Syscall callback
pub fn handle_syscall(proc: &mut Process, num: usize, arg0: usize, arg1: usize, arg2: usize, arg3: usize, arg4: usize, arg5: usize, arg6: usize) -> usize
{
    match num
    {
        // Exit Syscall
        60 =>
        {
            exit::syscall_exit(proc, arg0);
            0
        },
        default =>
        {
            kdebugln!(Syscalls, "Syscall from PID {}", proc.pid);
            kdebugln!(Syscalls, "Syscall {} ({}, {}, {}, {}, {}, {}, {})", default, arg0, arg1, arg2, arg3, arg4, arg5, arg6);
            0
        }
    }
}