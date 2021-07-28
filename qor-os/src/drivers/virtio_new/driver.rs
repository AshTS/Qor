use crate::*;

use super::consts::*;
use super::structs::*;

use alloc::format;

use VirtIOMmioOffsets as Field;

/// Generic VirtIO Device Driver
pub struct VirtIODeviceDriver
{
    device_type: VirtIODeviceType,
    device: VirtIOHelper
}

impl VirtIODeviceDriver
{
    /// Create a new driver arround a VirtIOHelper
    pub fn new(device_type: VirtIODeviceType, device: VirtIOHelper) -> Self
    {
        Self
        {
            device_type,
            device
        }
    }

    /// Initialize the VirtIO device driver
    pub fn init_driver(&self) -> Result<(), String>
    {
        Err(format!("Unimplemented"))
    }
}