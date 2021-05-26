use crate::*;

use super::InterruptContext;
use super::InterruptType;

/// Interrupt Handler
pub fn interrupt_handler(interrupt_context: InterruptContext) -> usize
{
    kdebugln!("{}", interrupt_context);

    match interrupt_context.get_cause()
    {
        InterruptType::MachineTimerInterrupt =>
        {
            // Prepare the timer for the next tick
            unsafe { drivers::TIMER_DRIVER.trigger() }
        },
        default => panic!("Unhandled Trap: {:?}", default)
    }

    interrupt_context.instruction_address()
}