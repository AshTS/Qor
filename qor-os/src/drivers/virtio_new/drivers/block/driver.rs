use crate::*;

use crate::drivers::virtio_new::*;

/// VirtIO Block Driver
pub struct BlockDriver
{
    device: VirtIODeviceDriver
}

impl BlockDriver
{
    /// Create a new block driver from a device driver
    pub fn new(device: VirtIODeviceDriver) -> Self
    {
        if device.get_device_type() != VirtIODeviceType::BlockDevice
        {
            panic!("Cannot create block device from {:?}", device.get_device_type());
        }

        Self
        {
            device
        }
    }

    /// Perform the device specific initialization
    pub fn device_specific(&mut self, _features: u32) -> Result<(), String>
    {
        Err(format!("Unable to perform device specific setup"))
    }
}