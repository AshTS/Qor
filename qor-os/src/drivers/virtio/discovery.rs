use crate::*;
use crate::drivers::virtio::MMIO_VIRTIO_MAGIC;

use super::VirtIODeviceDriver;
use super::consts::*;
use super::structs::*;

use alloc::format;

use VirtIOMmioOffsets as Field;

// VirtIO Address Space Slots for Devices
pub static mut VIRTIO_DEVICES: [Option<VirtIODeviceType>; 8] = [None, None, None, None, None, None, None, None];

/// Convert an index to an address
pub const fn virtio_index_to_address(index: usize) -> usize
{
    VIRT_IO_END + VIRT_IO_STEP * ((VIRT_IO_COUNT - index) - 1)
}

/// Convert an address to an index
pub const fn virtio_address_to_index(addr: usize) -> usize
{
    VIRT_IO_COUNT - (((addr - VIRT_IO_END) / VIRT_IO_STEP) + 1)
}

static_assertions::const_assert_eq!(virtio_address_to_index(virtio_index_to_address(0)), 0);
static_assertions::const_assert_eq!(virtio_address_to_index(virtio_index_to_address(1)), 1);
static_assertions::const_assert_eq!(virtio_address_to_index(virtio_index_to_address(7)), 7);

static_assertions::const_assert_eq!(virtio_index_to_address(virtio_address_to_index(VIRT_IO_END)), VIRT_IO_END);
static_assertions::const_assert_eq!(virtio_index_to_address(virtio_address_to_index(VIRT_IO_START)), VIRT_IO_START);

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
                VirtIODeviceType::UnknownDevice
            }
        };

        unsafe
        {
            VIRTIO_DEVICES[index] = Some(dev_type);
        }
    }
}

/// Get the driver at the given index
pub fn get_driver_at_index(index: usize) -> Option<VirtIODeviceDriver>
{
    unsafe { VIRTIO_DEVICES[index] }.map(
        |dt| 
        VirtIODeviceDriver::new(dt, 
            unsafe { VirtIOHelper::new(virtio_index_to_address(index)) }))
}

const FMT_ERROR: &'static str = "\x1B[31m";
const FMT_WARN: &'static str = "\x1B[33m";
const FMT_OK: &'static str = "\x1B[32m";
const FMT_CLEAR: &'static str = "\x1B[0m";

/// Initialize the located VirtIO devices
pub fn initialize_virtio_devices()
{
    let mut devices = DeviceCollection::new();

    for (i, dev_type) in unsafe {VIRTIO_DEVICES}.iter().enumerate()
    {
        if let Some(dev_type) = dev_type
        {
            let name = format!("{:?}", dev_type);
            kprint!("  Initializing (Device {}) {:15}.......... ", i, name);

            let mut driver = get_driver_at_index(i).unwrap();

            match dev_type
            {
                VirtIODeviceType::BlockDevice => 
                {
                    match driver.init_driver(!(1 << 5))
                    {
                        Err(e) =>
                        {
                            kprintln!("{}ERROR{}: `{}`", FMT_ERROR, FMT_CLEAR, e);
                        },
                        Ok(features) =>
                        {
                            let mut block_driver = super::drivers::block::BlockDriver::new(driver);

                            if let Err(e) = block_driver.device_specific(features)
                            {
                                kprintln!("{}ERROR{}: `{}`", FMT_ERROR, FMT_CLEAR, e);
                            }
                            else
                            {
                                kprintln!("{}OK{}", FMT_OK, FMT_CLEAR);
                                devices.block_devices.push(block_driver);
                            }
                        }
                    }
                },
                VirtIODeviceType::GPUDevice => 
                {
                    match driver.init_driver(!(1 << 5))
                    {
                        Err(e) =>
                        {
                            kprintln!("{}ERROR{}: `{}`", FMT_ERROR, FMT_CLEAR, e);
                        },
                        Ok(features) =>
                        {
                            let mut gpu_driver = super::drivers::gpu::GPUDriver::new(driver);

                            if let Err(e) = gpu_driver.device_specific(features)
                            {
                                kprintln!("{}ERROR{}: `{}`", FMT_ERROR, FMT_CLEAR, e);
                            }
                            else
                            {
                                kprintln!("{}OK{}", FMT_OK, FMT_CLEAR);
                                devices.gpu_devices.push(gpu_driver);
                            }
                        }
                    }
                },
                VirtIODeviceType::UnknownDevice => 
                {
                    kprintln!("{}Unknown Device{}", FMT_WARN, FMT_CLEAR);
                },
                _ =>
                {
                    kprintln!("{}No Initializer{}", FMT_WARN, FMT_CLEAR);
                }
            }
        }
    }

    *unsafe { &mut super::VIRTIO_DEVICE_COLLECTION } = Some(devices);
}