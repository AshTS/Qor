use crate::*;

use super::InterruptContext;
use super::InterruptType;

/// Interrupt Handler
pub fn interrupt_handler(interrupt_context: InterruptContext) -> usize
{
    kdebugln!(Interrupts, "{}", interrupt_context);

    match interrupt_context.get_cause()
    {
        InterruptType::MachineExternalInterrupt =>
        {
            // Get the next external interrupt
            if let Some(interrupt) = unsafe { drivers::PLIC_DRIVER.next_interrupt() }
            {
                // Run the handler
                super::extint::external_interrupt_handler(interrupt, &interrupt_context);

                // Complete the interrupt
                unsafe { drivers::PLIC_DRIVER.complete(interrupt) }; 
            }
        },
        InterruptType::MachineTimerInterrupt =>
        {
            // Prepare the timer for the next tick
            unsafe { drivers::TIMER_DRIVER.trigger() }
        },
        default => panic!("Unhandled Trap: {:?}", default)
    }

    interrupt_context.instruction_address()
}