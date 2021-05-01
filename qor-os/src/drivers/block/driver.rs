use crate::*;

use super::super::virtio::*;
use super::request::*;
use super::values::*;

const NUM_PAGES: usize = (core::mem::size_of::<VirtIOQueue>() + mem::pages::PAGE_SIZE - 1) / mem::pages::PAGE_SIZE;

/// Block Driver State
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockDriverState
{
    Uninitialized,
    Initialized
}

/// Block Device Driver
pub struct BlockDeviceDriver
{
    virtio_driver: VirtIODriver,
    queue: Option<Box<VirtIOQueue>>,
    state: BlockDriverState,
    device_index: usize,
    features: u32,
    descriptor_index: usize,
    acked_index: usize
}

impl BlockDeviceDriver
{
    /// Create a new block device driver
    pub fn new(mut virtio_driver: VirtIODriver) -> Self
    {
        if virtio_driver.check_device_id() != Some(VirtIODeviceID::BlockDevice)
        {
            panic!("Driver is not a block device");
        }

        let device_index = virtio_driver.get_index();

        Self
        {
            virtio_driver,
            queue: None,
            state: BlockDriverState::Uninitialized,
            device_index,
            features: 0,
            descriptor_index: 0,
            acked_index: 0
        }
    }

    /// Initialize the block device driver
    pub fn initialize(&mut self)
    {
        kdebugln!("Initializing Block Device Driver with index {}", self.device_index);

        if !self.virtio_driver.setup(
             |features| features & !(1 << 5) )
        {
            panic!("Unable to setup block device");
        }

        // Do Device Specific Here

        // Ensure the queue size is acceptable
        let qmax = self.virtio_driver.read_field(VirtIOOffsets::QueueNumMax);
        self.virtio_driver.write_field(VirtIOOffsets::QueueNum, VIRTIO_RING_SIZE as u32);
		if VIRTIO_RING_SIZE as u32 > qmax
        {
            panic!("The device does not support a ring size of {}", VIRTIO_RING_SIZE);
		}
        
        // We select the zeroth queue
        self.virtio_driver.write_field(VirtIOOffsets::QueueSel, 0);

        // Allocate the space for the queue
        let queue_ptr = mem::kpzalloc(NUM_PAGES) as *mut VirtIOQueue;

        // Set the page size and the pointer
        self.virtio_driver.write_field(VirtIOOffsets::GuestPageSize, mem::pages::PAGE_SIZE as u32);
        self.virtio_driver.write_field(VirtIOOffsets::QueuePfn, queue_ptr as u32 / mem::pages::PAGE_SIZE as u32);

        // Store the queue
        // Safety: This memory was just allocated by the kernel
        self.queue = Some(unsafe { Box::from_raw(queue_ptr) });

        // Finish initialization
        self.virtio_driver.driver_ok(VirtIOStatus::Acknowledge as u32 | VirtIOStatus::Driver as u32 | VirtIOStatus::FeaturesOk as u32);

        self.state = BlockDriverState::Initialized;
    }

    /// Insert the next descriptor in the ring buffer
    pub fn insert_descriptor(&mut self, descriptor: VirtIODescriptor) -> u16
    {
        assert!(self.state == BlockDriverState::Initialized);
        kdebugln!("Inserting Descriptor {:?}", descriptor);

        // Increment and handle wrapping
        self.descriptor_index += 1;
        self.descriptor_index %= VIRTIO_RING_SIZE;

        // Store the next flag for later
        let next = descriptor.flags & VirtIODescriptorFlags::Next as u16 > 0;

        // Insert the descriptor
        if let Some(queue) = &mut self.queue
        {
            queue.descriptors[self.descriptor_index] = descriptor;

            // Check the next flag, and if it is set, set the next value
            if next
            {
                queue.descriptors[self.descriptor_index].next = ((self.descriptor_index + 1) % VIRTIO_RING_SIZE) as u16;
            }
        }

        self.descriptor_index as u16
    }

    /// Generic block operation
    fn block_operation(&mut self, buffer: *mut u8, size: usize, offset: usize, write: bool)
    {
        let op = if write {"write to"} else {"read from"};

        kdebugln!(BlockDevice, "Attempting to {} device with index {}, size: {}, offset: {}", op, self.device_index, size, offset);

        // Get the sector number
        let sector = offset / 512;

        let mut request = Box::new(BlockDeviceRequest::new());

        let desc = VirtIODescriptor { addr:  &request.header as *const BlockDeviceRequestHeader as u64,
            len:   core::mem::size_of::<BlockDeviceRequestHeader>() as u32,
            flags: VirtIODescriptorFlags::Next as u16,
            next:  0, };

        let head_index = self.insert_descriptor(desc);

        request.header.sector = sector as u64;
        
        request.header.blktype = if write {VIRTIO_BLK_T_OUT} else {VIRTIO_BLK_T_IN};

        request.data.data = buffer;
        request.header.reserved = 0;
        request.status.status = 111;

        let desc = VirtIODescriptor { addr:  buffer as u64,
            len:   size as u32,
            flags: VirtIODescriptorFlags::Next as u16 | if !write {VirtIODescriptorFlags::Write as u16} else {0},
            next:  0, };

        let _data_index = self.insert_descriptor(desc);

        let desc = VirtIODescriptor { addr:  &request.status as *const BlockDeviceRequestStatus as u64,
            len:   core::mem::size_of::<BlockDeviceRequestStatus>() as u32,
            flags: VirtIODescriptorFlags::Write as u16,
            next:  0, };
        let _status_idx = self.insert_descriptor(desc);

        if let Some(queue) = &mut self.queue
        {
            queue.available.ring[queue.available.index as usize] = head_index;
            queue.available.index = (queue.available.index + 1) % VIRTIO_RING_SIZE as u16;
        }
        else
        {
            assert!(false)
        }

        self.virtio_driver.write_field(VirtIOOffsets::QueueNotify, 0);

        Box::leak(request);
    }

    /// Write to the block device
    pub fn write(&mut self, buffer: *mut u8, size: usize, offset: usize)
    {
        self.block_operation(buffer, size, offset, true)
    }

    /// Read from the block device
    pub fn read(&mut self, buffer: *mut u8, size: usize, offset: usize)
    {
        self.block_operation(buffer, size, offset, false)
    }

    /// To be triggered when an interrupt is recieved for this block device
    pub fn interrupt_trigger(&mut self)
    {
        if let Some(queue) = &mut self.queue
        {
            while self.acked_index != queue.used.index as usize
            {
                let ref element = queue.used.ring[self.acked_index];
                self.acked_index = (self.acked_index + 1) % VIRTIO_RING_SIZE;

                unsafe
                {
                    let _ = Box::from_raw(queue.descriptors[element.id as usize].addr as *mut BlockDeviceRequest);
                };
            }
        }
        else
        {
            kprintln!("Interrupt triggered on uninitialized block device at index {}", self.device_index);
        }
    }
}

impl core::ops::Drop for BlockDeviceDriver
{
    fn drop(&mut self)
    {
        // Leak the memory from the box
        if let Some(queue) = self.queue.take()
        {
            let ptr = Box::leak(queue);
            unsafe { mem::kpfree(ptr as *mut VirtIOQueue as *mut u8, NUM_PAGES) };
        }
        
    }
}