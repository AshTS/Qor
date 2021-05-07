use crate::*;

use trap::TrapFrame;

use super::syscalls;

/// General system call handler
pub fn syscall_handle(mepc: usize, frame: *mut TrapFrame, pid: Option<u16>) -> usize
{
    // Extract the syscall number as the first argument
    let syscall_number = unsafe { (*frame).regs[17] };

    let args = [
        unsafe { (*frame).regs[10] },
        unsafe { (*frame).regs[11] },
        unsafe { (*frame).regs[12] },
        unsafe { (*frame).regs[13] },
        unsafe { (*frame).regs[14] },
        unsafe { (*frame).regs[15] },
        unsafe { (*frame).regs[16] }
    ];

    if pid.is_none()
    {
        kprintln!("Syscall did not come from process `{}` ({}, {}, {}, {}, {}, {}, {})", syscall_number, args[0], args[1], args[2], args[3], args[4], args[5], args[6]);
    }

    if let Some(process) = process::get_process_manager().unwrap().process_by_pid_mut(pid.unwrap())
    {
        match syscall_number
        {
            10 => {syscalls::syscall_write(process, args[0])}
            60 => {syscalls::syscall_exit(process, args[0])},
            _ => { kprintln!("Got unknown system call `{}` ({}, {}, {}, {}, {}, {}, {}) from pid {:?}", syscall_number, args[0], args[1], args[2], args[3], args[4], args[5], args[6], pid); }
        }    
    }
    else
    {
        kprintln!("Syscall came from pid {}, which is not linked to a process`{}` ({}, {}, {}, {}, {}, {}, {})", syscall_number, args[0], args[1], args[2], args[3], args[4], args[5], args[6], pid.unwrap());
    }

    mepc + 4
}