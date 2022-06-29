// Include the drivers

pub mod mmio;
pub use mmio::*;

pub mod uart;
pub use uart::*;

/// UART Driver
/// 
/// Safety: The address is that given in the QEMU specification for the `virt` RISC-V board
pub static UART_DRIVER: UARTDriver = unsafe { UARTDriver::new( 0x1000_0000 ) };
