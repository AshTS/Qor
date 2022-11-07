// Location of VirtIO MMIO
pub const VIRT_IO_START: usize = 0x1000_8000;
pub const VIRT_IO_END: usize = 0x1000_1000;
pub const VIRT_IO_STEP: usize = 0x1000;

// Number of VirtIO Devices Available
pub const VIRT_IO_COUNT: usize = 1 + (VIRT_IO_START - VIRT_IO_END) / VIRT_IO_STEP;

// Magic value to denote a VirtIO Device
pub const MMIO_VIRTIO_MAGIC: u32 = 0x74_72_69_76;

// Status Flags
pub const VIRTIO_STATUS_ACKNOWLEDGE: u32 = 1;
pub const VIRTIO_STATUS_DRIVER: u32 = 2;
pub const VIRTIO_STATUS_FAILED: u32 = 128;
pub const VIRTIO_STATUS_FEATURES_OK: u32 = 8;
pub const VIRTIO_STATUS_DRIVER_OK: u32 = 4;
pub const VIRTIO_STATUS_DEVICE_NEEDS_RESET: u32 = 64;

// Allowable size of the Queues
pub const VIRTIO_QUEUE_SIZE: u32 = 1024;

// Descriptor flags
pub const VIRTIO_DESC_F_NEXT: u16 = 1;
pub const VIRTIO_DESC_F_WRITE: u16 = 2;
pub const VIRTIO_DESC_F_INDIRECT: u16 = 4;