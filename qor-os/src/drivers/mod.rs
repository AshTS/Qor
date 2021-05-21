//! Collection of all of the drivers for the kernel

// Modules for each driver
pub mod generic;
pub mod mmio;
pub mod uart;

// Static Driver Implementations
pub static mut UART_DRIVER: uart::UARTDriver = unsafe { uart::UARTDriver::new(0x1000_0000) };

/// Initialize the UART Driver
pub fn init_uart_driver()
{
    // Safety: This is safe as far as a race will lead to overlapping print outs
    unsafe { UART_DRIVER.init() };
}