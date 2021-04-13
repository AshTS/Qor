use crate::*;

/// Handler for external interrupts
pub fn external_interrupt_handler(id: drivers::plic::PLICInterrupt)
{

    match id
    {
        drivers::plic::PLICInterrupt::Interrupt10 => drivers::UART_DRIVER.callback(),
        default => kdebugln!(Interrupts, "Got an external interrupt: {:?}", default)
    }
}