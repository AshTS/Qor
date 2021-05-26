use crate::*;

use drivers::plic::PLICInterrupt;

/// External Interrupt Handler
pub fn external_interrupt_handler(interrupt: PLICInterrupt, _interrupt_context: &super::InterruptContext)
{
    kdebugln!(Interrupts, "ExtInt{}", interrupt.0);

    match interrupt
    {
        PLICInterrupt(10) => 
        {
            // Temporary handler to make sure the UART port is read
            use drivers::generic::ByteInterface;
            let c = unsafe { drivers::UART_DRIVER.read_byte()};

            match c
            {
                Some(10) | Some(13) => kprintln!(),
                Some(8) | Some(127) => kprint!("{} {}", 8 as char, 8 as char),
                Some(c) => kprint!("{}", c as char),
                _ => {}
            }
        },
        _ => {}
    }
}