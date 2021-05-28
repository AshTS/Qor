use crate::*;

use drivers::plic::PLICInterrupt;

/// External Interrupt Handler
pub fn external_interrupt_handler(interrupt: PLICInterrupt, _interrupt_context: &super::InterruptContext)
{
    kdebugln!(Interrupts, "ExtInt{}", interrupt.0);

    match interrupt
    {
        PLICInterrupt(1) => drivers::virtio::handle_interrupt(1),
        PLICInterrupt(2) => drivers::virtio::handle_interrupt(2),
        PLICInterrupt(3) => drivers::virtio::handle_interrupt(3),
        PLICInterrupt(4) => drivers::virtio::handle_interrupt(4),
        PLICInterrupt(5) => drivers::virtio::handle_interrupt(5),
        PLICInterrupt(6) => drivers::virtio::handle_interrupt(6),
        PLICInterrupt(7) => drivers::virtio::handle_interrupt(7),
        PLICInterrupt(8) => drivers::virtio::handle_interrupt(8),
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