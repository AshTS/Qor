//! Syscall Handling

use crate::*;

use process::process::Process;

// Modules
mod close;
mod execve;
mod exit;
mod open;
mod read;
mod write;

/// Syscall callback
pub fn handle_syscall(proc: &mut Process, num: usize, arg0: usize, arg1: usize, arg2: usize, arg3: usize, arg4: usize, arg5: usize, arg6: usize) -> usize
{
    match num
    {
        // Read Syscall
        0 =>
        {
            read::syscall_read(proc, arg0, arg1, arg2)
        },
        // Write Syscall
        1 =>
        {
            write::syscall_write(proc, arg0, arg1, arg2)
        },
        // Open Syscall
        2 =>
        {
            open::syscall_open(proc, arg0, arg1)
        },
        // Close Syscall
        3 =>
        {
            close::syscall_close(proc, arg0)
        },
        // Execve Syscall
        53 =>
        {
            execve::syscall_execve(proc, arg0)
        }
        // Exit Syscall
        60 =>
        {
            exit::syscall_exit(proc, arg0);
            0
        },
        default =>
        {
            kwarnln!("Syscall from PID {}", proc.pid);
            kwarnln!("Syscall {} ({}, {}, {}, {}, {}, {}, {})", default, arg0, arg1, arg2, arg3, arg4, arg5, arg6);
            0
        }
    }
}