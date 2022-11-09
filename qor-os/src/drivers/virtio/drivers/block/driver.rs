use crate::*;

use crate::drivers::virtio::*;

use super::consts::*;
use super::structs::*;

/// Block Driver Operation Handle
pub struct BlockOperation {
    request: *mut Request,
    semaphore: libutils::sync::semaphore::UpdateSemaphore<u8>
}

unsafe impl Send for BlockOperation {}
unsafe impl Sync for BlockOperation {}

impl libutils::sync::semaphore::Semaphore for BlockOperation {
    fn read(self) -> (bool, Option<Self>) {
        let (b, s) = self.semaphore.read();
        if let Some(s) = s {
            assert!(!b);

            (false, Some(BlockOperation { request: self.request, semaphore: s }))
        }
        else {
            unsafe { drop(alloc::boxed::Box::from_raw(self.request)) };

            (true, None)
        }
    }
}

impl BlockOperation {
    pub unsafe fn new(request: *mut Request) -> Self {
        Self {
            request,
            semaphore: libutils::sync::semaphore::UpdateSemaphore::new((request as *mut u8).add(STATUS_OFFSET), 111)
        }
    }
}

/// VirtIO Block Driver
pub struct BlockDriver {
    pub device: VirtIODeviceDriver,
}

impl BlockDriver {
    /// Create a new block driver from a device driver
    pub fn new(device: VirtIODeviceDriver) -> Self {
        if device.get_device_type() != VirtIODeviceType::BlockDevice {
            panic!(
                "Cannot create block device from {:?}",
                device.get_device_type()
            );
        }

        Self { device }
    }

    /// Perform the device specific initialization
    pub fn device_specific(&mut self, _features: u32) -> Result<(), alloc::string::String> {
        self.device.verify_queue_size()?;

        self.device.init_queues(1)?;

        self.device.driver_ok();

        Ok(())
    }

    /// Internal generic block driver
    fn block_operation(
        &mut self,
        buffer: *mut u8,
        size: u32,
        offset: u64,
        write: bool,
    ) -> Option<BlockOperation> {
        kdebugln!(unsafe BlockDevice, "Block Operation: Buffer: 0x{:x} {} bytes at offset 0x{:x} | {}", buffer as usize, size, offset, if write {"WRITE"} else {"READ"});

        let sector = offset / 512;
        // TODO: Before we get here, we are NOT allowed to schedule a read or
        // write OUTSIDE of the disk's size. So, we can read capacity from
        // the configuration space to ensure we stay within bounds.
        let blk_request = alloc::boxed::Box::leak(alloc::boxed::Box::new(Request::new()));

        let desc = VirtIODescriptor {
            addr: &(*blk_request).header as *const Header as u64,
            len: core::mem::size_of::<Header>() as u32,
            flags: VIRTIO_DESC_F_NEXT,
            next: 0,
        };

        let head_idx = self.device.add_descriptor_to_queue(0, desc);
        (*blk_request).header.sector = sector;

        // A write is an "out" direction, whereas a read is an "in" direction.
        (*blk_request).header.blktype = if true == write {
            VIRTIO_BLK_T_OUT
        } else {
            VIRTIO_BLK_T_IN
        };
        // We put 111 in the status. Whenever the device finishes, it will write into
        // status. If we read status and it is 111, we know that it wasn't written to by
        // the device.
        (*blk_request).data.data = buffer;
        (*blk_request).header.reserved = 0;
        (*blk_request).status.status = 111;
        let desc = VirtIODescriptor {
            addr: buffer as u64,
            len: size,
            flags: VIRTIO_DESC_F_NEXT
                | if false == write {
                    VIRTIO_DESC_F_WRITE
                } else {
                    0
                },
            next: 0,
        };

        let _data_idx = self.device.add_descriptor_to_queue(0, desc);

        let desc = VirtIODescriptor {
            addr: &(*blk_request).status as *const Status as u64,
            len: core::mem::size_of::<Status>() as u32,
            flags: VIRTIO_DESC_F_WRITE,
            next: 0,
        };

        let _status_idx = self.device.add_descriptor_to_queue(0, desc);

        self.device.send_on_queue(0, head_idx);

        let op = unsafe { BlockOperation::new(blk_request) };

        Some(op)
    }

    /// Send a read request to the block device
    pub unsafe fn read(&mut self, buffer: *mut u8, size: u32, offset: u64) -> Option<BlockOperation> {
        self.block_operation(buffer, size, offset, false)
    }

    /// Send a write request to the block device
    pub unsafe fn write(&mut self, buffer: *mut u8, size: u32, offset: u64) -> Option<BlockOperation> {
        self.block_operation(buffer, size, offset, true)
    }

    // Generic function to sync with a request finishing
    unsafe fn sync(mut request: BlockOperation) {
        
        use libutils::sync::semaphore::Semaphore;

        loop {
            if let (b, Some(v)) = request.read() {
                assert!(!b);
                request = v;
            }
            else {
                break;
            }
        }

        // while read_volatile(request.request).status.status == 111 {}
    }
    
    // Synchronously read from the disk into a pointer based buffer
    pub unsafe fn sync_read(&mut self, buffer: *mut u8, size: u32, offset: u64) {
        Self::sync(self.read(buffer, size, offset).unwrap());
    }

    // Synchronously write to disk from a pointer based buffer
    pub unsafe fn sync_write(&mut self, buffer: *mut u8, size: u32, offset: u64) {
        Self::sync(self.write(buffer, size, offset).unwrap());
    }

    pub fn sync_read_slice(&mut self, buffer: &mut [u8], offset: u64) -> Result<(), ()> {
        assert!(buffer.len() % 512 == 0);

        unsafe { self.sync_read(buffer.as_mut() as *mut [u8] as *mut u8, buffer.len() as u32, offset) }

        Ok(())
    }

    // Synchronously write to disk from a pointer based buffer
    pub fn sync_write_slice(&mut self, buffer: &[u8], offset: u64) -> Result<(), ()> {
        assert!(buffer.len() % 512 == 0);

        unsafe { self.sync_write(buffer.as_ref() as *const [u8] as *mut u8, buffer.len() as u32, offset) }

        Ok(())
    }
}
