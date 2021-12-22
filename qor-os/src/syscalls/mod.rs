//! Syscall Handling

use crate::*;

use process::process::Process;

// Modules
mod chdir;
mod close;
mod dup;
mod execve;
mod exit;
mod fork;
mod getcwd;
mod getdents;
mod getpid;
mod ioctl;
mod kill;
mod lseek;
mod mkdir;
mod mmap;
mod munmap;
mod nanosleep;
mod open;
mod pause;
mod pipe;
mod read;
mod setpgid;
mod sigaction;
mod sigreturn;
mod sync;
mod wait;
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
            open::syscall_open(proc, arg0, arg1, arg2)
        },
        // Close Syscall
        3 =>
        {
            close::syscall_close(proc, arg0)
        },
        // lseek Syscall
        8 =>
        {
            lseek::syscall_lseek(proc, arg0, arg1, arg2)
        },
        // mmap Syscall
        9 =>
        {
            mmap::syscall_mmap(proc, arg0, arg1, arg2, arg3, arg4, arg5)
        },
        // munmap Syscall
        11 =>
        {
            munmap::syscall_munmap(proc, arg0, arg1)
        },
        // sigaction Syscall
        13 =>
        {
            sigaction::syscall_sigaction(proc, arg0, arg1, arg2);
            0
        },
        // sigreturn Syscall
        15 =>
        {
            sigreturn::syscall_sigreturn(proc);
            0
        },
        // ioctl Syscall
        16 =>
        {
            ioctl::syscall_ioctl(proc, arg0, arg1, arg2)
        },
        // pipe Syscall
        22 =>
        {
            pipe::syscall_pipe(proc, arg0)
        },
        // dup Syscall
        32 =>
        {
            dup::syscall_dup(proc, arg0)
        },
        // dup2 Syscall
        33 =>
        {
            dup::syscall_dup2(proc, arg0, arg1)
        },
        // pause Syscall
        34 =>
        {
            pause::syscall_pause(proc)
        },
        // nanosleep Syscall
        35 =>
        {
            nanosleep::syscall_nanosleep(proc, arg0, arg1)
        },
        // getpid Syscall
        39 =>
        {
            getpid::syscall_getpid(proc)
        },
        // Fork Syscall
        57 =>
        {
            fork::syscall_fork(proc)
        }
        // Execve Syscall
        59 =>
        {
            execve::syscall_execve(proc, arg0, arg1, arg2)
        },
        // Exit Syscall
        60 =>
        {
            exit::syscall_exit(proc, arg0);
            0
        },
        // Wait Syscall
        61 =>
        {
            wait::syscall_wait(proc, arg0);
            0
        },
        // Kill Syscall
        62 =>
        {
            kill::syscall_kill(proc, arg0, arg1);
            0
        },
        // Getdents Syscall
        78 =>
        {
            getdents::syscall_getdents(proc, arg0, arg1, arg2)
        },
        // Getcwd Syscall
        79 =>
        {
            getcwd::syscall_getcwd(proc, arg0, arg1)
        },
        // Chdir Syscall
        80 =>
        {
            chdir::syscall_chdir(proc, arg0)
        },
        // Mkdir Syscall
        83 =>
        {
            mkdir::syscall_mkdir(proc, arg0, arg1)
        },
        // setpgid Syscall
        109 =>
        {
            setpgid::syscall_setpgid(proc, arg0, arg1)
        },
        // Sync Syscall
        162 =>
        {
            sync::syscall_sync(proc)
        },
        default =>
        {
            kwarnln!("Syscall from PID {}", proc.pid);
            kwarnln!("Syscall {} ({}, {}, {}, {}, {}, {}, {})", default, arg0, arg1, arg2, arg3, arg4, arg5, arg6);
            0
        }
    }
}