// Include the drivers

pub mod clint;
pub use clint::*;

pub mod mmio;
pub use mmio::*;

pub mod plic;
pub use plic::*;

pub mod uart;
pub use uart::*;

/// UART Driver
///
/// Safety: The address is that given in the QEMU specification for the `virt` RISC-V board
pub static UART_DRIVER: UARTDriver = unsafe { UARTDriver::new(0x1000_0000) };

/// PLIC driver
///
/// Safety: The address is that given in the QEMU specification for the `virt` RISC-V board
pub static PLIC_DRIVER: MMIOPlatformLevelInterruptController =
    unsafe { MMIOPlatformLevelInterruptController::new(0xc00_0000) };

/// CLINT driver
///
/// Safety: The address is that given in the QEMU specification for the `virt` RISC-V board
pub static CLINT_DRIVER: MMIOCoreLevelInterruptor =
    unsafe { MMIOCoreLevelInterruptor::new(0x200_0000) };

pub mod interrupts {
    use crate::drivers::InterruptID;

    pub const UART_INTERRUPT: InterruptID = 10;
}
