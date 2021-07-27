//! Generic VirtIO Driver

use crate::*;
use crate::drivers::virtio::MMIO_VIRTIO_MAGIC;

use super::consts::*;
use super::structs::*;

use VirtIOMmioOffsets as Field;

// VirtIO Address Space Slots for Devices
pub static mut VIRTIO_DEVICES: [Option<VirtIODeviceType>; 8] = [None, None, None, None, None, None, None, None];

/// Convert an index to an address
pub const fn virtio_index_to_address(index: usize) -> usize
{
    VIRT_IO_END + VIRT_IO_STEP * (index - 1)
}

/// Convert an address to an index
pub const fn virtio_address_to_index(addr: usize) -> usize
{
    ((addr - VIRT_IO_END) / VIRT_IO_STEP) + 1
}

/// Probe the VirtIO address space, discovering and logging any devices found,
/// will initialize the `VIRTIO_DEVICES` array
pub fn probe_virtio_address_space()
{
    assert!(VIRT_IO_START >= VIRT_IO_END);
    kdebugln!(VirtIO, "Probing VirtIO Devices");

    // The VirtIO Devices are laid out from VIRT_IO_END to VIRT_IO_START, at
    // VIRT_IO_STEP intervals, with indexing starting at 1

    for base_addr in  (VIRT_IO_END..=VIRT_IO_START).step_by(VIRT_IO_STEP)
    {
        let index = virtio_address_to_index(base_addr);
        kdebug!(VirtIO, "Probing VirtIO Device {} at 0x{:x}..........", index, base_addr);

        // Safety: This is directly from the VIRT_IO_START and VIRT_IO_END
        // values
        let dev = unsafe { VirtIOHelper::new(base_addr) };

        let magic = dev.read_field(Field::MagicValue);
        let id = dev.read_field(Field::DeviceId);

        // If the device is not a VirtIO device, move onto the next device
        if magic != MMIO_VIRTIO_MAGIC
        {
            kdebugln!(VirtIO, "Bad Device");

            continue;
        }

        // If the device id is 0, the device is disconnected
        if id == 0
        {
            kdebugln!(VirtIO, "Device Disconnected");
            continue;
        }

        // Match the id against the known VirtIO device ID's
        let dev_type = match id
        {
            1 =>
            {
                kdebugln!(VirtIO, "Network Card");
                VirtIODeviceType::NetworkCard
            },
            2 =>
            {
                kdebugln!(VirtIO, "Block Device");
                VirtIODeviceType::BlockDevice
            },
            3 =>
            {
                kdebugln!(VirtIO, "Console");
                VirtIODeviceType::Console
            },
            4 =>
            {
                kdebugln!(VirtIO, "Entropy Source");
                VirtIODeviceType::EntropySource
            },
            5 =>
            {
                kdebugln!(VirtIO, "Memory Ballooning");
                VirtIODeviceType::MemoryBallooning
            },
            6 =>
            {
                kdebugln!(VirtIO, "IO Memory");
                VirtIODeviceType::IOMemory
            },
            16 =>
            {
                kdebugln!(VirtIO, "GPU Device");
                VirtIODeviceType::GPUDevice
            },
            18 =>
            {
                kdebugln!(VirtIO, "Input Device");
                VirtIODeviceType::InputDevice
            }
            default =>
            {
                kdebugln!(VirtIO, "Unknown Device ({})", default);
                VirtIODeviceType::Unknown
            }
        };

        unsafe
        {
            VIRTIO_DEVICES[index - 1] = Some(dev_type);
        }
    }
}