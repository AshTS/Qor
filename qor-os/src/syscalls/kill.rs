use crate::*;

use crate::process::signals::POSIXSignal;
use crate::process::signals::SignalType;

/// Kill Syscall
pub fn syscall_kill(proc: &mut super::Process, pid: usize, signal: usize) -> usize
{
    // Convert the signal to the kernel's representation
    let sig_type = match signal
    {
        2 => SignalType::SIGINT,
        9 => SignalType::SIGKILL,
        15 => SignalType::SIGTERM,
        _ => { kwarnln!("Unknown signal {}", signal); return errno::EINVAL }
    };

    kdebugln!(Syscalls, "PID {} Sending Signal {:?} to PID {}", proc.pid, sig_type, pid);

    if pid != 0
    {
        if process::scheduler::get_process_manager().as_mut().unwrap().send_signal(
            pid as u16, 
            POSIXSignal::new(pid as u16, proc.pid, sig_type)).is_err()
        {
            errno::ESRCH // Bad pid
        }
        else
        {
            0
        }
    }
    else
    {
        if process::scheduler::get_process_manager().as_mut().unwrap().send_signal_group(
            proc.data.process_group_id,
            proc.pid,
            POSIXSignal::new(pid as u16, proc.pid, sig_type)).is_err()
        {
            errno::ESRCH // Bad pid
        }
        else
        {
            0
        }
    }
    
}