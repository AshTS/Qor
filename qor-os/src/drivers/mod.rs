// Include the drivers

pub mod block;
pub use block::*;

pub mod clint;
use alloc::sync::Arc;
pub use clint::*;

pub mod mmio;
use libutils::sync::InitThreadMarker;
pub use mmio::*;

pub mod plic;
pub use plic::*;

pub mod uart;
pub use uart::*;

use self::virtio::DeviceCollection;

pub mod virtio;

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

/// VIRTIO Driver Collection
///
static mut VIRTIO_DRIVER_COLLECTION: Option<Arc<DeviceCollection>> = None;

/// Handle virtio device discovery, we take the InitThreadMarker to make sure we don't alias the reference
pub fn virtio_device_discovery(_marker: InitThreadMarker) -> Result<(), alloc::string::String> {
    let devices = virtio::discover_virtio_devices()?;

    // Safety: We have the single thread marker, so this reference will never alias
    unsafe { VIRTIO_DRIVER_COLLECTION.replace(Arc::new(devices)) };

    Ok(())
}

/// Get a reference to the virtio device collection
pub fn virtio_device_collection() -> Arc<DeviceCollection> {
    // Safety: This value can never be updated after the initial initialization, thus it is safe to get a shared reference to it.
    if let Some(collection) = unsafe { &VIRTIO_DRIVER_COLLECTION } {
        collection.clone()
    } else {
        panic!("device collection not yet initialized");
    }
}
