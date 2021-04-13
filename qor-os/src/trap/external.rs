use crate::*;

/// Handler for external interrupts
pub fn external_interrupt_handler(id: u32)
{
    kdebugln!(Interrupts, "Got an external interrupt: {}", id);
}