// Include the drivers

pub mod mmio;
pub use mmio::*;

pub mod power;
pub use power::*;

pub mod uart;
pub use uart::*;

/// Power Driver
/// 
/// Safety: The address is that given in the QEMU specification for the `virt` RISC-V board
pub static POWER_DRIVER: PowerDriver = unsafe { PowerDriver::new(0x10_0000) };

/// UART Driver
///
/// Safety: The address is that given in the QEMU specification for the `virt` RISC-V board
pub static UART_DRIVER: UARTDriver = unsafe { UARTDriver::new(0x1000_0000) };
