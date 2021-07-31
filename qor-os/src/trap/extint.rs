use crate::*;

use drivers::plic::PLICInterrupt;

// UART Input Raw Ring Buffer
static mut UART_IN_BUFFER: utils::ByteRingBuffer = utils::ByteRingBuffer::new();

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
            // use drivers::generic::ByteInterface;
            // let c = unsafe { drivers::UART_DRIVER.read_byte()};

            drivers::get_uart_driver().notify_recieve();

            /*
            match c
            {
                Some(10) | Some(13) => 
                {
                    kprintln!();

                    unsafe { UART_IN_BUFFER.enqueue_byte(10) };

                    // Move the line from the ring buffer to the stdin buffer
                    while let Some(byte) = unsafe { UART_IN_BUFFER.dequeue_byte() }
                    {
                        unsafe 
                        {
                            process::descriptor::STDIN_BUFFER.enqueue_byte(byte);
                        }
                    }
                },
                Some(8) | Some(127) =>
                {
                    kprint!("{} {}", 8 as char, 8 as char);

                    unsafe { UART_IN_BUFFER.pop_byte() };
                },
                Some(c) =>
                {
                    kprint!("{}", c as char);

                    unsafe { UART_IN_BUFFER.enqueue_byte(c) };
                },
                _ => {}
            } */
        },
        _ => {}
    }
}