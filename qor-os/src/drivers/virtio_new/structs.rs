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
    base: usize
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