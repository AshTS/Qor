pub mod block;
pub mod plic;
mod timer;
mod uart;
pub mod virtio;

use crate::kprintln;

lazy_static::lazy_static!
{
    // Safety: The QEMU emulator has a UART mmio interface at 0x1000_0000
    pub static ref UART_DRIVER: uart::UartDriver =  unsafe{uart::UartDriver::new(0x1000_0000)};

    // Safety: The QEMU emulator has a CLINT mmio interface at 0x200_0000
    pub static ref TIMER_DRIVER: timer::TimerDriver = unsafe{timer::TimerDriver::new(0x200_0000)};

    // Safety: The QEMU emulator has a PLIC mmio interface at 0xc00_0000
    pub static ref PLIC_DRIVER: plic::PLICDriver = unsafe{plic::PLICDriver::new(0xc00_0000)};
}

/// Get a uart driver
pub fn get_uart_driver() -> uart::UartDriver
{
    unsafe { uart::UartDriver::new(0x1000_0000) }
}

/// Initialize the UART driver
pub fn init_uart_driver()
{
    UART_DRIVER.initialize();

    kprintln!("UART Driver Initialized");
}

/// Initialize the PLIC Driver
pub fn init_plic_driver()
{
    PLIC_DRIVER.set_threshold(plic::PLICPriority::Priority0);
    PLIC_DRIVER.enable_interrupt(plic::PLICInterrupt::Interrupt10);
    PLIC_DRIVER.set_priority(plic::PLICInterrupt::Interrupt10, plic::PLICPriority::Priority1);

    kprintln!("Initializing PLIC");

}