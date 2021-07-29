use crate::*;

use super::consts::*;
use super::structs::*;

use alloc::format;

use crate::mem::PAGE_SIZE;

use VirtIOMmioOffsets as Field;

/// Generic VirtIO Device Driver
pub struct VirtIODeviceDriver
{
    device_type: VirtIODeviceType,
    device: VirtIOHelper,
    device_status: u32,
    driver_ready: bool,
    queues: Vec<*mut VirtIOQueue>,
    queue_aux_data: Vec<AuxQueueData>
}

impl VirtIODeviceDriver
{
    /// Create a new driver arround a VirtIOHelper
    pub fn new(device_type: VirtIODeviceType, device: VirtIOHelper) -> Self
    {
        Self
        {
            device_type,
            device,
            device_status: 0,
            driver_ready: false,
            queues: Vec::new(),
            queue_aux_data: Vec::new()
        }
    }

    /// Finalize device initialization
    pub fn driver_ok(&mut self)
    {
        self.device_status |= VIRTIO_STATUS_DRIVER_OK;
        self.device.write_field(Field::Status, self.device_status);
        self.driver_ready = true;
    }

    /// Fail the device initialization
    fn fail(&mut self)
    {
        self.device.write_field(Field::Status, VIRTIO_STATUS_FAILED);
        self.driver_ready = false;
    }

    /// If needed, verity the queue size with the device, for now, if the device
    /// does not accept our global queue size, we will simply fail, it may be
    /// possible in the future to have variable queue sizes, however, for now it
    /// is easiest to just have a single global value, and set it such that all
    /// needed devices accept it
    pub fn verify_queue_size(&self) -> Result<(), String>
    {
        let largest_allowable_queue = self.device.read_field(Field::QueueNumMax);
        self.device.write_field(Field::QueueNum, VIRTIO_QUEUE_SIZE);

        if VIRTIO_QUEUE_SIZE > largest_allowable_queue
        {
            Err(format!("Max queue size {} < {}", largest_allowable_queue, VIRTIO_QUEUE_SIZE))
        }
        else
        {
            Ok(())
        }
    }

    /// Initialize some number of queues for the device
    pub fn init_queues(&mut self, queue_count: usize) -> Result<(), String>
    {
        self.queues = Vec::with_capacity(queue_count);
        self.queue_aux_data = Vec::with_capacity(queue_count);

        // Notify the device of our page size
        self.device.write_field(Field::GuestPageSize, PAGE_SIZE as u32);

        for queue_index in 0..queue_count
        {
            const PAGE_COUNT: usize = (core::mem::size_of::<VirtIOQueue>() + PAGE_SIZE - 1) / PAGE_SIZE;
            let queue_location = crate::mem::kpalloc(PAGE_COUNT, "VirtIO Device Queue").unwrap() as *mut VirtIOQueue;

            // Tell the device which queue we are telling it about
            self.device.write_field(Field::QueueSel, queue_index as u32);

            // Tell the device where this queue is to be stored in memory
            self.device.write_field(Field::QueuePfn, (queue_location as usize / PAGE_SIZE) as u32);

            self.queues.push(queue_location);

            // Add the auxillary queue data 
            let data = AuxQueueData
            {
                index: 0,
                ack_index: 0
            };

            self.queue_aux_data.push(data);
        }

        Ok(())
    }

    /// Internal VirtIO device driver initialization, should be called wrapped
    /// in an error handler which will set the failed bit to notify the device
    /// of the failure
    fn wrapped_init(&mut self, accepted_features: u32) -> Result<u32, String>
    {
        /* From the spec (https://docs.oasis-open.org/virtio/virtio/v1.1/cs01/virtio-v1.1-cs01.html) 3.1.1

            1. Reset the device.
            2. Set the ACKNOWLEDGE status bit: the guest OS has noticed the
                device.
            3. Set the DRIVER status bit: the guest OS knows how to drive the 
                device.
            4. Read device feature bits, and write the subset of feature bits 
                understood by the OS and driver to the device. During this step 
                the driver MAY read (but MUST NOT write) the device-specific
                configuration fields to check that it can support the device
                before accepting it.
            5. Set the FEATURES_OK status bit. The driver MUST NOT accept new
                feature bits after this step.
            6. Re-read device status to ensure the FEATURES_OK bit is still set:
                otherwise, the device does not support our subset of features
                and the device is unusable.
            7. Perform device-specific setup, including discovery of virtqueues
                for the device, optional per-bus setup, reading and possibly
                writing the device’s virtio configuration space, and population
                of virtqueues.
            8. Set the DRIVER_OK status bit. At this point the device is “live”.

        */

        /*
            This function will bring us through step 6, after that, it is up to
            creating the specific driver to handle the remainder of the
            initialization.
        */

        // 1. Reset the device.
        self.device_status = 0;
        self.device.write_field(Field::Status, self.device_status);

        // 2. Set the ACKNOWLEDGE status bit
        self.device_status |= VIRTIO_STATUS_ACKNOWLEDGE;
        self.device.write_field(Field::Status, self.device_status);

        // 3. Set the DRIVER status bit
        self.device_status |= VIRTIO_STATUS_DRIVER;
        self.device.write_field(Field::Status, self.device_status);

        // 4. Read device feature bits
        let features = self.device.read_field(Field::HostFeatures);

        // , and write the subset of feature bits understood by the OS and
        // driver to the device
        self.device.write_field(Field::GuestFeatures, features & accepted_features);

        // 5. Set the FEATURES_OK status bit
        self.device_status |= VIRTIO_STATUS_FEATURES_OK;
        self.device.write_field(Field::Status, self.device_status);

        // 6. Re-read device status to ensure the FEATURES_OK bit is still set
        let status = self.device.read_field(Field::Status);
        if status & VIRTIO_STATUS_FEATURES_OK == 0
        {
            return Err(format!("Device Refuse Features"));
        }
        
        Ok(features & accepted_features)
    }

    /// Initialize the VirtIO device driver, returns the features the device
    /// accepted
    pub fn init_driver(&mut self, accepted_features: u32) -> Result<u32, String>
    {
        match self.wrapped_init(accepted_features)
        {
            Ok(v) => Ok(v),
            Err(e) =>
            {
                self.fail();

                Err(e)
            }
        }
    }

    /// Get the device type
    pub fn get_device_type(&self) -> VirtIODeviceType
    {
        self.device_type
    }
}