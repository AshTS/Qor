use crate::*;

use super::consts::*;

use crate::mem::PAGE_SIZE;

/// VirtIO Memory Mapped Input / Output Offsets
#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtIOMmioOffsets
{
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

/// Types of VirtIO Devices
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtIODeviceType
{
    NetworkCard = 1,
    BlockDevice = 2,
    Console = 3,
    EntropySource = 4,
    MemoryBallooning = 5,
    IOMemory = 6,
    GPUDevice = 16,
    InputDevice = 18,
    UnknownDevice = 0
}

/// Helper structure for interacting with a VirtIO device
pub struct VirtIOHelper
{
    pub base: usize
}

impl VirtIOHelper
{
    /// Create a new VirtIOHelper
    ///
    /// Safety: The given base address must be a valid VirtIO device base
    /// address
    pub unsafe fn new(base: usize) -> Self
    {
        Self
        {
            base
        }
    }

    /// Read from a field in the VirtIO device
    pub fn read_field(&self, field: VirtIOMmioOffsets) -> u32
    {
        use crate::drivers::mmio::read_offset;

        // Safety: See safety of VirtIOHelper constructor
        unsafe { read_offset(self.base, field as usize) }
    }

    /// Write to a field in the VirtIO device
    pub fn write_field(&self, field: VirtIOMmioOffsets, value: u32)
    {
        use crate::drivers::mmio::write_offset;

        // Safety: See safety of VirtIOHelper constructor
        unsafe { write_offset(self.base, field as usize, value) }
    }
}

/* From the VirtIO page on the OsDev Wiki (https://wiki.osdev.org/Virtio)
    struct VirtualQueue
    {
    struct Buffers[QueueSize]
    {
        uint64_t Address; // 64-bit address of the buffer on the guest machine.
        uint32_t Length;  // 32-bit length of the buffer.
        uint16_t Flags;   // 1: Next field contains linked buffer index;  2: Buffer is write-only (clear for read-only).
                        // 4: Buffer contains additional buffer addresses.
        uint16_t Next;    // If flag is set, contains index of next buffer in chain.
    }
    
    struct Available
    {
        uint16_t Flags;             // 1: Do not trigger interrupts.
        uint16_t Index;             // Index of the next ring index to be used.  (Last available ring buffer index+1)
        uint16_t [QueueSize] Ring;  // List of available buffer indexes from the Buffers array above.
        uint16_t EventIndex;        // Only used if VIRTIO_F_EVENT_IDX was negotiated
    }
    
    uint8_t[] Padding;  // Reserved
    // 4096 byte alignment
    struct Used
    {
        uint16_t Flags;            // 1: Do not notify device when buffers are added to available ring.
        uint16_t Index;            // Index of the next ring index to be used.  (Last used ring buffer index+1)
        struct Ring[QueueSize]
        {
        uint32_t Index;  // Index of the used buffer in the Buffers array above.
        uint32_t Length; // Total bytes written to buffer.
        }
        uint16_t AvailEvent;       // Only used if VIRTIO_F_EVENT_IDX was negotiated
    }
    }
 */

 #[repr(C)]
pub struct VirtIODescriptor
{
	pub addr:  u64,
	pub len:   u32,
	pub flags: u16,
	pub next:  u16,
}

#[repr(C)]
pub struct VirtIOAvailable
{
	pub flags: u16,
	pub idx:   u16,
	pub ring:  [u16; VIRTIO_QUEUE_SIZE as usize],
	pub event: u16,
}

#[repr(C)]
pub struct VirtIOUsedElem 
{
	pub id:  u32,
	pub len: u32,
}

#[repr(C)]
pub struct VirtIOUsed
{
	pub flags: u16,
	pub idx:   u16,
	pub ring:  [VirtIOUsedElem; VIRTIO_QUEUE_SIZE as usize],
	pub event: u16,
}

const BEFORE_PADDING: usize = core::mem::size_of::<VirtIODescriptor>() * VIRTIO_QUEUE_SIZE as usize + 
                                core::mem::size_of::<VirtIOAvailable>();
const PADDING: usize = (PAGE_SIZE - BEFORE_PADDING % PAGE_SIZE) % PAGE_SIZE;

#[repr(C, align(4096))]
pub struct VirtIOQueue
{
	pub desc:  [VirtIODescriptor; VIRTIO_QUEUE_SIZE as usize],
	pub avail: VirtIOAvailable,
	pub padding0: [u8; PADDING],
	pub used:     VirtIOUsed,
}

static_assertions::const_assert!((BEFORE_PADDING + PADDING) % PAGE_SIZE == 0);
static_assertions::const_assert_eq!(core::mem::align_of::<VirtIOQueue>() % PAGE_SIZE, 0);

/// Auxillary queue data for the VirtIO driver
pub struct AuxQueueData
{
    pub index: usize,
    pub ack_index: usize
}

/// VirtIO Devices Collection
pub struct DeviceCollection
{
    pub block_devices: Vec<super::drivers::block::BlockDriver>
}

impl DeviceCollection
{
    /// Create a new, empty device collection
    pub fn new() -> Self
    {
        Self
        {
            block_devices: Vec::new()
        }
    }
}