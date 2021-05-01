use crate::*;

/// Handler for external interrupts
pub fn external_interrupt_handler(id: drivers::plic::PLICInterrupt)
{
    match id
    {
        drivers::plic::PLICInterrupt::Interrupt1 => drivers::virtio::handle_interrupt(1),
        drivers::plic::PLICInterrupt::Interrupt2 => drivers::virtio::handle_interrupt(2),
        drivers::plic::PLICInterrupt::Interrupt3 => drivers::virtio::handle_interrupt(3),
        drivers::plic::PLICInterrupt::Interrupt4 => drivers::virtio::handle_interrupt(4),
        drivers::plic::PLICInterrupt::Interrupt5 => drivers::virtio::handle_interrupt(5),
        drivers::plic::PLICInterrupt::Interrupt6 => drivers::virtio::handle_interrupt(6),
        drivers::plic::PLICInterrupt::Interrupt7 => drivers::virtio::handle_interrupt(7),
        drivers::plic::PLICInterrupt::Interrupt8 => drivers::virtio::handle_interrupt(8),
        drivers::plic::PLICInterrupt::Interrupt10 => drivers::UART_DRIVER.callback(),
        default => kdebugln!(Interrupts, "Got an external interrupt: {:?}", default)
    }
}