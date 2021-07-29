use crate::*;

use crate::drivers::virtio::*;

use super::structs::*;
use super::consts::*;

/// VirtIO GPU Driver
pub struct GPUDriver
{
    pub device: VirtIODeviceDriver
}

impl GPUDriver
{
    /// Create a new gpu driver from a device driver
    pub fn new(device: VirtIODeviceDriver) -> Self
    {
        if device.get_device_type() != VirtIODeviceType::GPUDevice
        {
            panic!("Cannot create GPU device from {:?}", device.get_device_type());
        }

        Self
        {
            device
        }
    }

    /// Perform the device specific initialization
    pub fn device_specific(&mut self, _features: u32) -> Result<(), String>
    {
        Err(format!("Not yet implemented"))
    }
}