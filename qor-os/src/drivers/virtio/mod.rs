//! Virtual IO Device Handling
use crate::*;

// QEMU VirtIO Device Locations
pub const VIRT_IO_START: usize = 0x1000_8000;
pub const VIRT_IO_END: usize = 0x1000_1000;
pub const VIRT_IO_STEP: usize = 0x1000;

// Set the ring size
pub const VIRTIO_RING_SIZE: usize = 1 << 10;

pub const MMIO_VIRTIO_MAGIC: u32 = 0x74_72_69_76;

// Flags
// Descriptor flags have VIRTIO_DESC_F as a prefix
// Available flags have VIRTIO_AVAIL_F

pub const VIRTIO_DESC_F_NEXT: u16 = 1;
pub const VIRTIO_DESC_F_WRITE: u16 = 2;
pub const VIRTIO_DESC_F_INDIRECT: u16 = 4;

pub const VIRTIO_AVAIL_F_NO_INTERRUPT: u16 = 1;

pub const VIRTIO_USED_F_NO_NOTIFY: u16 = 1;


// VirtIO structures

// The descriptor holds the data that we need to send to 
// the device. The address is a physical address and NOT
// a virtual address. The len is in bytes and the flags are
// specified above. Any descriptor can be chained, hence the
// next field, but only if the F_NEXT flag is specified.
#[repr(C)]
pub struct Descriptor {
	pub addr:  u64,
	pub len:   u32,
	pub flags: u16,
	pub next:  u16,
}

#[repr(C)]
pub struct Available {
	pub flags: u16,
	pub idx:   u16,
	pub ring:  [u16; VIRTIO_RING_SIZE],
	pub event: u16,
}

#[repr(C)]
pub struct UsedElem {
	pub id:  u32,
	pub len: u32,
}

#[repr(C)]
pub struct Used {
	pub flags: u16,
	pub idx:   u16,
	pub ring:  [UsedElem; VIRTIO_RING_SIZE],
	pub event: u16,
}

#[repr(C)]
pub struct Queue {
	pub desc:  [Descriptor; VIRTIO_RING_SIZE],
	pub avail: Available,
	// Calculating padding, we need the used ring to start on a page boundary. We take the page size, subtract the
	// amount the descriptor ring takes then subtract the available structure and ring.
	pub padding0: [u8; mem::PAGE_SIZE * 8 - core::mem::size_of::<Descriptor>() * VIRTIO_RING_SIZE - core::mem::size_of::<Available>()],
	pub used:     Used,
}

// The MMIO transport is "legacy" in QEMU, so these registers represent
// the legacy interface.
#[repr(usize)]
pub enum MmioOffsets {
	MagicValue = 0x000,
	Version = 0x004,
	DeviceId = 0x008,
	VendorId = 0x00c,
	HostFeatures = 0x010,
	HostFeaturesSel = 0x014,
	GuestFeatures = 0x020,
	GuestFeaturesSel = 0x024,
	GuestPageSize = 0x028,
	QueueSel = 0x030,
	QueueNumMax = 0x034,
	QueueNum = 0x038,
	QueueAlign = 0x03c,
	QueuePfn = 0x040,
	QueueNotify = 0x050,
	InterruptStatus = 0x060,
	InterruptAck = 0x064,
	Status = 0x070,
	Config = 0x100,
}

// Enumerations in Rust aren't easy to convert back
// and forth. Furthermore, we're going to use a u32
// pointer, so we need to "undo" the scaling that
// Rust will do with the .add() function.
impl MmioOffsets {
	pub fn val(self) -> usize {
		self as usize
	}

	pub fn scaled(self, scale: usize) -> usize {
		self.val() / scale
	}

	pub fn scale32(self) -> usize {
		self.scaled(4)
	}
}

pub enum StatusField {
	Acknowledge = 1,
	Driver = 2,
	Failed = 128,
	FeaturesOk = 8,
	DriverOk = 4,
	DeviceNeedsReset = 64,
}

// The status field will be compared to the status register. So,
// I've made some helper functions to checking that register easier.
impl StatusField {
	pub fn val(self) -> usize {
		self as usize
	}

	pub fn val32(self) -> u32 {
		self as u32
	}

	pub fn test(sf: u32, bit: StatusField) -> bool {
		sf & bit.val32() != 0
	}

	pub fn is_failed(sf: u32) -> bool {
		StatusField::test(sf, StatusField::Failed)
	}

	pub fn needs_reset(sf: u32) -> bool {
		StatusField::test(sf, StatusField::DeviceNeedsReset)
	}

	pub fn driver_ok(sf: u32) -> bool {
		StatusField::test(sf, StatusField::DriverOk)
	}

	pub fn features_ok(sf: u32) -> bool {
		StatusField::test(sf, StatusField::FeaturesOk)
	}
}

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceTypes
{
	None = 0,
	Network = 1,
	Block = 2,
	Console = 3,
	Entropy = 4,
	Gpu = 16,
	Input = 18,
	Memory = 24,
}


pub struct VirtioDevice
{
	pub devtype: DeviceTypes,
}

impl VirtioDevice
{
	pub const fn new() -> Self
    {
		VirtioDevice { devtype: DeviceTypes::None, }
	}

	pub const fn new_with(devtype: DeviceTypes) -> Self {
		VirtioDevice { devtype }
	}
}


pub static mut VIRTIO_DEVICES: [Option<VirtioDevice>; 8] = [None, None, None, None, None, None, None, None];

/// Probe the VirtIO address space
pub fn probe_virt_io()
{
    assert!(VIRT_IO_START >= VIRT_IO_END);

    kdebugln!(VirtIO, "Probing VirtIO Devices");

    let mut base = VIRT_IO_START;

    while base >= VIRT_IO_END
    {
        kdebug!(VirtIO, "Probing VirtIO Device at 0x{:x} ..........", base);

        let magic = unsafe { super::mmio::read_offset::<u32>(base, 0) };
        let id = unsafe { super::mmio::read_offset::<u32>(base, 8) };
        
        if magic == MMIO_VIRTIO_MAGIC
        {
            let t = match id
            {
                0 =>
                {
                    kdebugln!(VirtIO, "Empty Device Slot");
                    VirtioDevice::new_with(DeviceTypes::None)
                },
                2 =>
                {
                    kdebug!(VirtIO, "Block Device... ");

                    if crate::drivers::block::setup_block_device(base as *mut u32)
                    {
                        kdebugln!(VirtIO, "Success");
                    }
                    else
                    {
                        kdebugln!(VirtIO, "Failure");
                    }

                    VirtioDevice::new_with(DeviceTypes::Block)
                },
                device_id => {
                    kdebugln!(VirtIO, "Unknown Device Id {}", device_id);
                    VirtioDevice::new_with(DeviceTypes::None)
                }
            };

            let idx = (base - VIRT_IO_END) / VIRT_IO_STEP;

            if t.devtype != DeviceTypes::None
            {
                unsafe { VIRTIO_DEVICES[idx] = Some(t) };
            }
        }
        else
        {
            kdebugln!(VirtIO, "Not a VirtIO Device");
        }

        base -= VIRT_IO_STEP;
    }
}

pub fn handle_interrupt(interrupt: u32)
{
	let idx = interrupt as usize - 1;
    if let Some(vd) = unsafe { &VIRTIO_DEVICES[idx] }
    {
        match vd.devtype
        {
            DeviceTypes::Block =>
            {
                drivers::block::handle_interrupt(idx);
            },
            _ =>
            {
                kprintln!("Invalid device generated interrupt!");
            },
        }
    }
    else
    {
        kprintln!("Spurious interrupt {}", interrupt);
    }
	
}

pub fn init_virtio_interrupts()
{
    for i in 1..9
    {
        unsafe { drivers::PLIC_DRIVER.enable(drivers::plic::PLICInterrupt(i)) };
        unsafe { drivers::PLIC_DRIVER.set_priority(drivers::plic::PLICInterrupt(i), 
                                        drivers::plic::PLICPriority::Priority1) };
    }
}