use crate::*;

use super::ops;

// QEMU VirtIO Device Locations
const VIRT_IO_START: usize = 0x1000_8000;
const VIRT_IO_END: usize = 0x1000_1000;
const VIRT_IO_STEP: usize = 0x1000;

// Set the ring size
pub const VIRTIO_RING_SIZE: usize = 1 << 7;

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

/// VirtIO Status Register Values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum VirtIOStatus
{
    Acknowledge = 1,
    Driver = 2,
    DriverOk = 4,
    FeaturesOk = 8,
    DeviceNeedsReset = 64,
    Failed = 128
}

/// VirtIO Descriptor Flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum VirtIODescriptorFlags
{
    Next = 1,
    Write = 2,
    Indirect = 4
}

/// VirtIO Available Flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum VirtIOAvailableFlags
{
    NoInterrupt = 1
}

/// VirtIO Used Flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum VirtIOUsedFlags
{
    Used = 1
}

/// VirtIO Descriptor
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtIODescriptor
{
	pub addr:  u64,
	pub len:   u32,
	pub flags: u16,
	pub next:  u16,
}

/// VirtIO Available
#[repr(C)]
pub struct VirtIOAvailable
{
	pub flags: u16,
	pub index:   u16,
	pub ring:  [u16; VIRTIO_RING_SIZE],
	pub event: u16,
}

/// VirtIO Used Element
#[repr(C)]
pub struct VirtIOUsedElem
{
	pub id:  u32,
	pub len: u32,
}

/// VirtIO Used Ring
#[repr(C)]
pub struct VirtIOUsed
{
	pub flags: u16,
	pub index:   u16,
	pub ring:  [VirtIOUsedElem; VIRTIO_RING_SIZE],
	pub event: u16,
}

const PADDING_SIZE: usize = mem::pages::PAGE_SIZE - core::mem::size_of::<VirtIODescriptor>() * VIRTIO_RING_SIZE - core::mem::size_of::<VirtIOAvailable>();

/// VirtIO Ring
#[repr(C)]
pub struct VirtIOQueue
{
	pub descriptors:  [VirtIODescriptor; VIRTIO_RING_SIZE],
	pub available: VirtIOAvailable,
    // Pad the used ring to be on a page boundary
	pub padding0: [u8; PADDING_SIZE],
	pub used:     VirtIOUsed,
}

/// VirtIO Device Driver
pub struct VirtIODriver
{
    base: usize,
    device_id: Option<Option<VirtIODeviceID>>
}

impl VirtIODriver
{
    /// Create a new VirtIO Device Driver
    /// Safety: The base address must be the base address of a Virtual IO Device
    pub unsafe fn new(base: usize) -> Self
    {
        Self
        {
            base,
            device_id: None
        }
    }

    /// Verify the magic
    pub fn verify_magic(&self) -> bool
    {
        // Safety: The base address must be a valid base address of a Virtual IO Device
        unsafe { ops::verify_magic(self.base) }
    }

    /// Check the device ID
    pub fn check_device_id(&mut self) -> Option<VirtIODeviceID>
    {
        if let Some(device_id) = self.device_id
        {
            device_id
        }
        else
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
            let device_id = match id
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
            };

            self.device_id = Some(device_id);
            device_id
        }

        
    }

    /// Read a field from the device
    pub fn read_field(&self, field: ops::VirtIOOffsets) -> u32
    {
        kdebugln!("VIRTIO Reading field {:?}", field);
        unsafe { ops::virtio_read_u32(self.base, field) }
    }

    /// Write a field to the device
    pub fn write_field(&self, field: ops::VirtIOOffsets, data: u32)
    {
        kdebugln!("VIRTIO Writing field {} -> {:?}", data, field);
        unsafe { ops::virtio_write_u32(self.base, field, data) }
    }

    /// Get the index of the virtio device
    pub fn get_index(&self) -> usize
    {
        (self.base - VIRT_IO_END) / VIRT_IO_STEP
    }

    /// Set the device to a failed state
    pub fn fail(&mut self, prev: u32)
    {
        self.write_field(ops::VirtIOOffsets::Status, prev | VirtIOStatus::Failed as u32);
    }

    /// Enable the driver ok flag
    pub fn driver_ok(&mut self, prev: u32)
    {
        self.write_field(ops::VirtIOOffsets::Status, prev | VirtIOStatus::DriverOk as u32);
    }

    /// Setup the device
    pub fn setup(&mut self, review_features: impl Fn(u32) -> u32) -> bool
    {
        let index = self.get_index();

        kdebugln!(VirtIO, "Performing Setup on Device {}", index);

        // Reset the device by writing 0 to the status register.
        let mut status = 0;
        self.write_field(ops::VirtIOOffsets::Status, status);

        // Set the ACKNOWLEDGE status bit to the status register.
        status |= VirtIOStatus::Acknowledge as u32;
        self.write_field(ops::VirtIOOffsets::Status, status);

        // Set the DRIVER status bit to the status register.
        status |= VirtIOStatus::Driver as u32;
        self.write_field(ops::VirtIOOffsets::Status, status);

        // Read device features from host_features register.
        let features = self.read_field(ops::VirtIOOffsets::HostFeatures);

        // Negotiate the set of features and write what you'll accept to guest_features register.
        self.write_field(ops::VirtIOOffsets::GuestFeatures, review_features(features));

        // Set the FEATURES_OK status bit to the status register.
        status |= VirtIOStatus::FeaturesOk as u32;
        self.write_field(ops::VirtIOOffsets::Status, status);

        // Re-read the status register to confirm that the device accepted your features.
        let result = self.read_field(ops::VirtIOOffsets::Status);

        kprintln!("Features: 0x{:x}", result);

        if result & VirtIOStatus::FeaturesOk as u32 == 0
        {
            kdebugln!(VirtIO, "Feature Negotiation Failed");

            // Terminate the initialization
            self.fail(status);
            false
        }
        else
        {
            true
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
        let mut driver = unsafe { VirtIODriver::new(base) };

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
