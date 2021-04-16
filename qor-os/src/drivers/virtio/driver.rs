use crate::*;

use super::ops;

// QEMU VirtIO Device Locations
const VIRT_IO_START: usize = 0x1000_8000;
const VIRT_IO_END: usize = 0x1000_1000;
const VIRT_IO_STEP: usize = 0x1000;

/// VirtIO Device ID's
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtIODeviceID
{
    NetworkCard,
    BlockDevice,
    Console,
    EntropySource,
    MemoryBallooning,
    IOMemory,
    RPMSG,
    SCSIHost,
    Transport9P,
    MAC802_11WLAN
}

/// VirtIO Device Driver
pub struct VirtIODriver
{
    base: usize
}

impl VirtIODriver
{
    /// Create a new VirtIO Device Driver
    /// Safety: The base address must be the base address of a Virtual IO Device
    pub unsafe fn new(base: usize) -> Self
    {
        Self
        {
            base
        }
    }

    /// Verify the magic
    pub fn verify_magic(&self) -> bool
    {
        // Safety: The base address must be a valid base address of a Virtual IO Device
        unsafe { ops::verify_magic(self.base) }
    }

    /// Check the device ID
    pub fn check_device_id(&self) -> Option<VirtIODeviceID>
    {
        // Get the device ID
        // Safety: The base address must be a valid base address of a Virtual IO Device
        let id = unsafe { ops::virtio_read_u32(self.base, ops::VirtIOOffsets::DeviceId) };

        // From the VirtIO page on the OS Dev Wiki
        // 1    Network Card
        // 2    Block Device
        // 3    Console
        // 4    Entropy Source
        // 5    Memory Ballooning
        // 6    IO Memory
        // 7    RPMSG
        // 8    SCSI Host
        // 9    9P Transport
        // 10   MAC802.11 WLAN  
        match id
        {
            0 => None, // This is reserved, but generally ignored
            1 => Some(VirtIODeviceID::NetworkCard),
            2 => Some(VirtIODeviceID::BlockDevice),
            3 => Some(VirtIODeviceID::Console),
            4 => Some(VirtIODeviceID::EntropySource),
            5 => Some(VirtIODeviceID::MemoryBallooning),
            6 => Some(VirtIODeviceID::IOMemory),
            7 => Some(VirtIODeviceID::RPMSG),
            8 => Some(VirtIODeviceID::SCSIHost),
            9 => Some(VirtIODeviceID::Transport9P),
            10 => Some(VirtIODeviceID::MAC802_11WLAN),
            _ => None
        }
    }
}

/// Probe the VirtIO address space
pub fn probe_virt_io() -> Vec<VirtIODriver>
{
    assert!(VIRT_IO_START >= VIRT_IO_END);

    kdebugln!(VirtIO, "Probing VirtIO Devices");

    let mut result = Vec::with_capacity((VIRT_IO_START - VIRT_IO_END) / VIRT_IO_STEP + 1);

    let mut base = VIRT_IO_START;

    while base >= VIRT_IO_END
    {
        // Safety: The addresses are hardcoded to be those of the VirtIO Devices
        let driver = unsafe { VirtIODriver::new(base) };

        if driver.verify_magic()
        {
            if let Some(id) = driver.check_device_id()
            {
                kdebugln!(VirtIO, "Found {:?} at base address 0x{:x}", id, base);

                result.push(driver);
            }
        }

        base -= VIRT_IO_STEP;
    }

    result
}
