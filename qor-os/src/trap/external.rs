use crate::*;

/// Handler for external interrupts
pub fn external_interrupt_handler(id: drivers::plic::PLICInterrupt)
{
    kdebugln!(Interrupts, "Got an external interrupt: {:?}", id);
}