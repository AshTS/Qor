use crate::*;

use process::signals::*;

/// sigaction Syscall
pub fn syscall_sigaction(proc: &mut super::Process, signal: usize, new_ptr: usize, old_ptr: usize)
{
    let new_ref = unsafe { (proc.map_mem(new_ptr).unwrap() as *mut SignalAction).as_mut() };
    let _old_ref = unsafe { (proc.map_mem(old_ptr).unwrap() as *mut SignalAction).as_mut() };

    if let Some(new) = new_ref
    {
        let sig = SignalType::number_to_signal(signal);

        kdebugln!(Signals, "sigaction from PID {}: On Signal {:?}", proc.pid, sig);

        if new.flags & 1 > 0
        {
            // Set the handler as a function
            proc.data.signal_map.insert(sig, SignalDisposition::Handler(new.action_fn_ptr));
        }
        else
        {
            // Set the handler as a dispoisiton
            match new.handler_value
            {
                0 => {},
                1 => { return; },
                2 => { proc.data.signal_map.insert(sig, SignalDisposition::Ignore); return; }
                _ => todo!()
            }
        }

        // TODO: We still need to handle writing to the old_ref if it exists
    }
    else
    {
        // We still need to work on writing to the old_ref, and this branch is
        // where that is the only option.
        todo!()
    }
}