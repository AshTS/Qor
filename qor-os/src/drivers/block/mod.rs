use crate::drivers::{
    virtio,
    virtio::{Descriptor, MmioOffsets, Queue, StatusField, VIRTIO_RING_SIZE}};
use core::mem::size_of;

use crate::*;

#[repr(C)]
pub struct Header
{
    blktype:  u32,
    reserved: u32,
    sector:   u64,
}

impl Header
{
    pub fn new() -> Self
    {
        Self
        {
            blktype: 0,
            reserved: 0,
            sector: 0,
        }
    }
}

#[repr(C)]
pub struct Data
{
    data: *mut u8,
}

impl Data
{
    pub fn new() -> Self
    {
        Self
        {
            data: 0 as *mut u8
        }
    }
}

#[repr(C)]
pub struct Status
{
    status: u8,
}

impl Status
{
    pub fn new() -> Self
    {
        Self
        {
            status: 0
        }
    }
}

#[repr(C)]
pub struct Request
{
    header: Header,
    data:   Data,
    status: Status,
    head:   u16,
}

impl Request
{
    pub fn new() -> Self
    {
        Self
        {
            header: Header::new(),
            data: Data::new(),
            status: Status::new(),
            head: 0,
        }
    }
}

pub struct BlockDevice
{
    queue:        *mut Queue,
    dev:          *mut u32,
    idx:          u16,
    ack_used_idx: u16,
    read_only:    bool,
}

// Type values
pub const VIRTIO_BLK_T_IN: u32 = 0;
pub const VIRTIO_BLK_T_OUT: u32 = 1;
pub const VIRTIO_BLK_T_FLUSH: u32 = 4;
pub const VIRTIO_BLK_T_DISCARD: u32 = 11;
pub const VIRTIO_BLK_T_WRITE_ZEROES: u32 = 13;

// Status values
pub const VIRTIO_BLK_S_OK: u8 = 0;
pub const VIRTIO_BLK_S_IOERR: u8 = 1;
pub const VIRTIO_BLK_S_UNSUPP: u8 = 2;

// Feature bits
pub const VIRTIO_BLK_F_SIZE_MAX: u32 = 1;
pub const VIRTIO_BLK_F_SEG_MAX: u32 = 2;
pub const VIRTIO_BLK_F_GEOMETRY: u32 = 4;
pub const VIRTIO_BLK_F_RO: u32 = 5;
pub const VIRTIO_BLK_F_BLK_SIZE: u32 = 6;
pub const VIRTIO_BLK_F_FLUSH: u32 = 9;
pub const VIRTIO_BLK_F_TOPOLOGY: u32 = 10;
pub const VIRTIO_BLK_F_CONFIG_WCE: u32 = 11;
pub const VIRTIO_BLK_F_DISCARD: u32 = 13;
pub const VIRTIO_BLK_F_WRITE_ZEROES: u32 = 14;

// Much like with processes, Rust requires some initialization
// when we declare a static. In this case, we use the Option
// value type to signal that the variable exists, but not the
// queue itself. We will replace this with an actual queue when
// we initialize the block system.
static mut BLOCK_DEVICES: [Option<BlockDevice>; 8] = [None, None, None, None, None, None, None, None];

pub fn setup_block_device(ptr: *mut u32) -> bool
{
    unsafe
    {
        // We can get the index of the device based on its address.
        // 0x1000_1000 is index 0
        // 0x1000_2000 is index 1
        // ...
        // 0x1000_8000 is index 7
        // To get the number that changes over, we shift right 12 places (3 hex digits)
        let idx = (ptr as usize - virtio::VIRT_IO_END) >> 12;
        // [Driver] Device Initialization
        // 1. Reset the device (write 0 into status)

        // ptr.add(MmioOffsets::Status.scale32()).write_volatile(0);
        crate::mmio::mmio_write_int(ptr as usize, MmioOffsets::Status as usize, 0);

        let mut status_bits = StatusField::Acknowledge.val32();
        // 2. Set ACKNOWLEDGE status bit
        crate::mmio::mmio_write_int(ptr as usize, MmioOffsets::Status as usize, status_bits);
        // 3. Set the DRIVER status bit
        status_bits |= StatusField::Driver.val32();
        crate::mmio::mmio_write_int(ptr as usize, MmioOffsets::Status as usize, status_bits);
        // 4. Read device feature bits, write subset of feature
        // bits understood by OS and driver    to the device.
        // let host_features = ptr.add(MmioOffsets::HostFeatures.scale32()).read_volatile();
        let host_features = crate::mmio::mmio_read_int(ptr as usize, MmioOffsets::HostFeatures as usize);
        let guest_features = host_features & !(1 << VIRTIO_BLK_F_RO);
        let ro = host_features & (1 << VIRTIO_BLK_F_RO) != 0;
        crate::mmio::mmio_write_int(ptr as usize, MmioOffsets::GuestFeatures as usize, guest_features);
        // 5. Set the FEATURES_OK status bit
        status_bits |= StatusField::FeaturesOk.val32();
        crate::mmio::mmio_write_int(ptr as usize, MmioOffsets::Status as usize, status_bits);
        // 6. Re-read status to ensure FEATURES_OK is still set.
        // Otherwise, it doesn't support our features.
        let status_ok = crate::mmio::mmio_read_int(ptr as usize, MmioOffsets::Status as usize);
        // If the status field no longer has features_ok set,
        // that means that the device couldn't accept
        // the features that we request. Therefore, this is
        // considered a "failed" state.
        if false == StatusField::features_ok(status_ok)
        {
            kprint!("features fail...");
            
            crate::mmio::mmio_write_int(ptr as usize, MmioOffsets::Status as usize, StatusField::Failed as u32);
            return false;
        }
        // 7. Perform device-specific setup.
        // Set the queue num. We have to make sure that the
        // queue size is valid because the device can only take
        // a certain size.
        let qnmax = crate::mmio::mmio_read_int(ptr as usize, MmioOffsets::QueueNumMax as usize);
        crate::mmio::mmio_write_int(ptr as usize, MmioOffsets::QueueNum as usize, VIRTIO_RING_SIZE as u32);
        if VIRTIO_RING_SIZE as u32 > qnmax
        {
            kprint!("queue size fail...");
            return false;
        }

        let num_pages = (size_of::<Queue>() + 4096 - 1) / 4096;

        ptr.add(MmioOffsets::QueueSel.scale32()).write_volatile(0);
        crate::mmio::mmio_write_int(ptr as usize, MmioOffsets::QueueSel as usize, 0);


        let queue_ptr = mem::kpzalloc(num_pages) as *mut Queue;
        let queue_pfn = queue_ptr as u32;
        crate::mmio::mmio_write_int(ptr as usize, MmioOffsets::GuestPageSize as usize, 4096 as u32);

        crate::mmio::mmio_write_int(ptr as usize, MmioOffsets::QueuePfn as usize, queue_pfn / 4096 as u32);


        let bd = BlockDevice { queue:        queue_ptr,
                            dev:          ptr,
                            idx:          0,
                            ack_used_idx: 0,
                            read_only:    ro, };
        BLOCK_DEVICES[idx] = Some(bd);


        status_bits |= StatusField::DriverOk.val32();
        crate::mmio::mmio_write_int(ptr as usize, MmioOffsets::Status as usize, status_bits);

        true
    }
}

pub fn fill_next_descriptor(bd: &mut BlockDevice, desc: Descriptor) -> u16
{
    unsafe
    {
        bd.idx = (bd.idx + 1) % VIRTIO_RING_SIZE as u16;
        (*bd.queue).desc[bd.idx as usize] = desc;
        if (*bd.queue).desc[bd.idx as usize].flags & virtio::VIRTIO_DESC_F_NEXT != 0
        {
            (*bd.queue).desc[bd.idx as usize].next = (bd.idx + 1) % VIRTIO_RING_SIZE as u16;
        }
        bd.idx
    }
}


pub fn block_op(dev: usize, buffer: *mut u8, size: u32, offset: u64, write: bool)
{
    unsafe 
    {
        if let Some(bdev) = BLOCK_DEVICES[dev - 1].as_mut()
        {
            // Check to see if we are trying to write to a read only device.
            if true == bdev.read_only && true == write {
                kdebugln!(BlockDevice, "Trying to write to read/only!");
                return;
            }
            let sector = offset / 512;
            // TODO: Before we get here, we are NOT allowed to schedule a read or
            // write OUTSIDE of the disk's size. So, we can read capacity from
            // the configuration space to ensure we stay within bounds.
            let blk_request = Box::leak(Box::new(Request::new()));
            
            let desc = Descriptor { addr:  &(*blk_request).header as *const Header as u64,
                                    len:   size_of::<Header>() as u32,
                                    flags: virtio::VIRTIO_DESC_F_NEXT,
                                    next:  0, };
            let head_idx = fill_next_descriptor(bdev, desc);
            (*blk_request).header.sector = sector;
            // A write is an "out" direction, whereas a read is an "in" direction.
            (*blk_request).header.blktype = if true == write {
                VIRTIO_BLK_T_OUT
            }
            else {
                VIRTIO_BLK_T_IN
            };
            // We put 111 in the status. Whenever the device finishes, it will write into
            // status. If we read status and it is 111, we know that it wasn't written to by
            // the device.
            (*blk_request).data.data = buffer;
            (*blk_request).header.reserved = 0;
            (*blk_request).status.status = 111;
            let desc = Descriptor { addr:  buffer as u64,
                                    len:   size,
                                    flags: virtio::VIRTIO_DESC_F_NEXT
                                        | if false == write {
                                            virtio::VIRTIO_DESC_F_WRITE
                                        }
                                        else {
                                            0
                                        },
                                    next:  0, };
            let _data_idx = fill_next_descriptor(bdev, desc);
            let desc = Descriptor { addr:  &(*blk_request).status as *const Status as u64,
                                    len:   size_of::<Status>() as u32,
                                    flags: virtio::VIRTIO_DESC_F_WRITE,
                                    next:  0, };
            let _status_idx = fill_next_descriptor(bdev, desc);
            (*bdev.queue).avail.ring[(*bdev.queue).avail.idx as usize] = head_idx;
            (*bdev.queue).avail.idx = ((*bdev.queue).avail.idx + 1) % virtio::VIRTIO_RING_SIZE as u16;
            // The only queue a block device has is 0, which is the request
            // queue.
            bdev.dev.add(MmioOffsets::QueueNotify.scale32()).write_volatile(0);
            crate::mmio::mmio_write_int(bdev.dev as usize, MmioOffsets::QueueNotify as usize, 0)
        }
    }
}

pub fn read(dev: usize, buffer: *mut u8, size: u32, offset: u64)
{
    block_op(dev, buffer, size, offset, false);
}

pub fn write(dev: usize, buffer: *mut u8, size: u32, offset: u64)
{
    block_op(dev, buffer, size, offset, true);
}


pub fn pending(bd: &mut BlockDevice)
{
    unsafe
    {
        let ref queue = *bd.queue;
        while bd.ack_used_idx != queue.used.idx
        {
            let ref elem = queue.used.ring[bd.ack_used_idx as usize];
            bd.ack_used_idx = (bd.ack_used_idx + 1) % VIRTIO_RING_SIZE as u16;
            let rq = queue.desc[elem.id as usize].addr as *const Request;
            Box::from_raw(rq as *mut u8);
        }
    }
}

pub fn handle_interrupt(idx: usize)
{
    unsafe
    {
        if let Some(bdev) = BLOCK_DEVICES[idx].as_mut()
        {
            kdebugln!(BlockDevice, "Block Device Interrupt {}", idx);
            pending(bdev);
        }
        else
        {
            kdebugln!(BlockDevice, "Invalid block device for interrupt {}", idx + 1);
        }
    }
}

pub struct BlockDeviceDriver(usize);

impl BlockDeviceDriver
{
    pub fn read(&self, buffer: *mut u8, size: u32, offset: u64)
    {
        read(self.0, buffer, size, offset);
    }

    pub fn write(&self, buffer: *mut u8, size: u32, offset: u64)
    {
        write(self.0, buffer, size, offset);
    }
}

/// Get the Block Device Driver Struct for the given index (the index is among
/// the connected block devices)
pub fn get_driver_by_index(orig_index: usize) -> BlockDeviceDriver
{
    let mut index = orig_index;
    for (i, v) in unsafe { &BLOCK_DEVICES }.iter().enumerate().rev()
    {
        if index == 0 && v.is_some()
        {
            return BlockDeviceDriver(i + 1);
        }
        else if v.is_some()
        {
            index -= 1;
        }
    }

    panic!("Cannot get block driver with index {}", orig_index);
}