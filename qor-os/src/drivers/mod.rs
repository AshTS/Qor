//! Collection of all of the drivers for the kernel

use crate::*;

// Modules for each driver
pub mod generic;
pub mod gpu;
pub mod mmio;
pub mod plic;
pub mod rtc;
pub mod timer;
pub mod uart;
pub mod virtio;

// Static Driver Implementations
pub static mut PLIC_DRIVER: plic::PLICDriver = unsafe { plic::PLICDriver::new(0x0c00_0000) };
pub static mut TIMER_DRIVER: timer::TimerDriver = unsafe { timer::TimerDriver::new(0x200_0000) };
pub static mut UART_DRIVER: uart::UARTDriver = unsafe { uart::UARTDriver::new(0x1000_0000) };

/// Initialize the UART Driver
pub fn init_uart_driver()
{
    // Safety: This is safe as far as a race will lead to overlapping print outs
    unsafe { UART_DRIVER.init() };
}

/// Initialize the Timer Driver (set the given frequency)
pub fn init_timer_driver(freq: usize)
{
    kdebugln!(Initialization, "Setting timer frequency to {}Hz", freq);

    unsafe
    {
        drivers::TIMER_DRIVER.set_frequency(freq);
        drivers::TIMER_DRIVER.trigger();
    }
}

/// Initialize the PLIC Driver (enable the UART receive interrupt)
pub fn init_plic_driver()
{
    unsafe { drivers::PLIC_DRIVER.set_threshold(drivers::plic::PLICPriority::Disable) };
    unsafe { drivers::PLIC_DRIVER.enable(drivers::plic::PLICInterrupt(10)) };
    unsafe { drivers::PLIC_DRIVER.set_priority(drivers::plic::PLICInterrupt(10), 
                                       drivers::plic::PLICPriority::Priority1) };
}

/// Get the UART driver
pub fn get_uart_driver() -> &'static mut uart::UARTDriver
{
    unsafe { &mut UART_DRIVER }
}