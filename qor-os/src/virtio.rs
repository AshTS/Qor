// virtio.rs
// VirtIO routines for the VirtIO protocol
// Stephen Marz
// 10 March 2020

use crate::{block, block::setup_block_device};
use core::mem::size_of;

use crate::*;

// Flags
// Descriptor flags have VIRTIO_DESC_F as a prefix
// Available flags have VIRTIO_AVAIL_F

pub const VIRTIO_DESC_F_NEXT: u16 = 1;
pub const VIRTIO_DESC_F_WRITE: u16 = 2;
pub const VIRTIO_DESC_F_INDIRECT: u16 = 4;

pub const VIRTIO_AVAIL_F_NO_INTERRUPT: u16 = 1;

pub const VIRTIO_USED_F_NO_NOTIFY: u16 = 1;

// According to the documentation, this must be a power
// of 2 for the new style. So, I'm changing this to use
// 1 << instead because that will enforce this standard.
pub const VIRTIO_RING_SIZE: usize = 1 << 7;

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
	pub padding0: [u8; 4096 - size_of::<Descriptor>() * VIRTIO_RING_SIZE - size_of::<Available>()],
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

#[repr(usize)]
pub enum DeviceTypes {
	None = 0,
	Network = 1,
	Block = 2,
	Console = 3,
	Entropy = 4,
	Gpu = 16,
	Input = 18,
	Memory = 24,
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

// We probably shouldn't put these here, but it'll help
// with probing the bus, etc. These are architecture specific
// which is why I say that.
pub const MMIO_VIRTIO_START: usize = 0x1000_1000;
pub const MMIO_VIRTIO_END: usize = 0x1000_8000;
pub const MMIO_VIRTIO_STRIDE: usize = 0x1000;
pub const MMIO_VIRTIO_MAGIC: u32 = 0x74_72_69_76;

// The VirtioDevice is essentially a structure we can put into an array
// to determine what virtio devices are attached to the system. Right now,
// we're using the 1..=8  linearity of the VirtIO devices on QEMU to help
// with reducing the data structure itself. Otherwise, we might be forced
// to use an MMIO pointer.
pub struct VirtioDevice {
	pub devtype: DeviceTypes,
}

impl VirtioDevice {
	pub const fn new() -> Self {
		VirtioDevice { devtype: DeviceTypes::None, }
	}

	pub const fn new_with(devtype: DeviceTypes) -> Self {
		VirtioDevice { devtype }
	}
}

static mut VIRTIO_DEVICES: [Option<VirtioDevice>; 8] = [None, None, None, None, None, None, None, None];

/// Probe the VirtIO bus for devices that might be
/// out there.
pub fn probe() {
	// Rust's for loop uses an Iterator object, which now has a step_by
	// modifier to change how much it steps. Also recall that ..= means up
	// to AND including MMIO_VIRTIO_END.
	for addr in (MMIO_VIRTIO_START..=MMIO_VIRTIO_END).step_by(MMIO_VIRTIO_STRIDE) {
		kprint!("Virtio probing 0x{:08x}...", addr);
		let magicvalue;
		let deviceid;
		let ptr = addr as *mut u32;
		unsafe {
			magicvalue = ptr.read_volatile();
			deviceid = ptr.add(2).read_volatile();
		}
		// 0x74_72_69_76 is "virt" in little endian, so in reality
		// it is triv. All VirtIO devices have this attached to the
		// MagicValue register (offset 0x000)
		if MMIO_VIRTIO_MAGIC != magicvalue {
			kprintln!("not virtio.");
		}
		// If we are a virtio device, we now need to see if anything
		// is actually attached to it. The DeviceID register will
		// contain what type of device this is. If this value is 0,
		// then it is not connected.
		else if 0 == deviceid {
			kprintln!("not connected.");
		}
		// If we get here, we have a connected virtio device. Now we have
		// to figure out what kind it is so we can do device-specific setup.
		else {
			match deviceid {
				// DeviceID 1 is a network device
				1 => {
					kprint!("network device...");
					if false == setup_network_device(ptr) {
						kprintln!("setup failed.");
					}
					else {
						kprintln!("setup succeeded!");
					}
				},
				// DeviceID 2 is a block device
				2 => {
					kprint!("block device...");
					if false == setup_block_device(ptr) {
						kprintln!("setup failed.");
					}
					else {
						let idx = (addr - MMIO_VIRTIO_START) >> 12;
						unsafe {
							VIRTIO_DEVICES[idx] =
								Some(VirtioDevice::new_with(DeviceTypes::Block));
						}
						kprintln!("setup succeeded!");
					}
				},
				_ => kprintln!("unknown device type."),
			}
		}
	}
}

pub fn setup_network_device(_ptr: *mut u32) -> bool {
	false
}

pub fn setup_gpu_device(_ptr: *mut u32) -> bool {
	false
}

pub fn setup_input_device(_ptr: *mut u32) -> bool {
	false
}

// The External pin (PLIC) trap will lead us here if it is
// determined that interrupts 1..=8 are what caused the interrupt.
// In here, we try to figure out where to direct the interrupt
// and then handle it.
pub fn handle_interrupt(interrupt: u32) {
	let idx = interrupt as usize - 1;
	unsafe {
		if let Some(vd) = &VIRTIO_DEVICES[idx] {
			match vd.devtype {
				DeviceTypes::Block => {
					block::handle_interrupt(idx);
				},
				_ => {
					kprintln!("Invalid device generated interrupt!");
				},
			}
		}
		else {
			kprintln!("Spurious interrupt {}", interrupt);
		}
	}
}