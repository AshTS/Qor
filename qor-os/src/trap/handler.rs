use crate::*;

use super::TrapFrame;

/// Trap handler (only called from the trap handler in assembly)
#[no_mangle]
extern "C" fn m_trap(epc: usize,
                     _tval: usize,
                     _cause: usize,
                     _hart: usize,
                     _status: usize,
                     _frame: &'static mut TrapFrame)
                     -> usize
{
    kprintln!("Interrupt!");

    epc
}
