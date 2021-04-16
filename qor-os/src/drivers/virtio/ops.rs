use crate::mmio;

/// VirtIO MMIO Offsets
#[repr(usize)]
pub enum VirtIOOffsets
{
    MagicNumber = 0x000,
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

/// Read a u8 from a VirtIO Offset
/// Safety: The base address must be a valid address to a VirtIO Device
pub unsafe fn virtio_read_u8(base: usize, offset: VirtIOOffsets) -> u8
{
    mmio::mmio_read_byte(base, offset as usize)
}

/// Read a u16 from a VirtIO Offset
/// Safety: The base address must be a valid address to a VirtIO Device
pub unsafe fn virtio_read_u16(base: usize, offset: VirtIOOffsets) -> u16
{
    mmio::mmio_read_short(base, offset as usize)
}

/// Read a u32 from a VirtIO Offset
/// Safety: The base address must be a valid address to a VirtIO Device
pub unsafe fn virtio_read_u32(base: usize, offset: VirtIOOffsets) -> u32
{
    mmio::mmio_read_int(base, offset as usize)
}

/// Read a u64 from a VirtIO Offset
/// Safety: The base address must be a valid address to a VirtIO Device
pub unsafe fn virtio_read_u64(base: usize, offset: VirtIOOffsets) -> u64
{
    mmio::mmio_read_long(base, offset as usize)
}

/// Write a u8 to a VirtIO Offset
/// Safety: The base address must be a valid address to a VirtIO Device
pub unsafe fn virtio_write_u8(base: usize, offset: VirtIOOffsets, data: u8)
{
    mmio::mmio_write_byte(base, offset as usize, data);
}

/// Write a u16 to a VirtIO Offset
/// Safety: The base address must be a valid address to a VirtIO Device
pub unsafe fn virtio_write_u16(base: usize, offset: VirtIOOffsets, data: u16)
{
    mmio::mmio_write_short(base, offset as usize, data);
}

/// Write a u32 to a VirtIO Offset
/// Safety: The base address must be a valid address to a VirtIO Device
pub unsafe fn virtio_write_u32(base: usize, offset: VirtIOOffsets, data: u32)
{
    mmio::mmio_write_int(base, offset as usize, data);
}

/// Write a u64 to a VirtIO Offset
/// Safety: The base address must be a valid address to a VirtIO Device
pub unsafe fn virtio_write_u64(base: usize, offset: VirtIOOffsets, data: u64)
{
    mmio::mmio_write_long(base, offset as usize, data);
}

/// Verify the magic value
/// Safety: The base address must be a valid address to a VirtIO Device
pub unsafe fn verify_magic(base: usize) -> bool
{
    virtio_read_u32(base, VirtIOOffsets::MagicNumber) == 0x74_72_69_76
}