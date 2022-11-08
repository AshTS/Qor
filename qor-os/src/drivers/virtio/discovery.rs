use crate::*;
use libutils::sync::Mutex;

use super::*;

pub fn discover_virtio_devices() -> Result<DeviceCollection, alloc::string::String> {
    let mut collection = DeviceCollection::new();

    let mut address = VIRT_IO_START;

    while address >= VIRT_IO_END {
        let device = unsafe { VirtIOHelper::new(address) };
        let device_type = unsafe { core::mem::transmute::<u32, VirtIODeviceType>(device.read_field(VirtIOMmioOffsets::DeviceId)) };
        let device_driver = VirtIODeviceDriver::new(device_type, device);
        
        collection.add_device(device_driver)?;
        
        address -= VIRT_IO_STEP;
    }

    Ok(collection)
}

impl DeviceCollection {
    pub fn add_device(&mut self, mut device_driver: VirtIODeviceDriver) -> Result<(), alloc::string::String> {
        match device_driver.get_device_type() {
            // Initialize the device driver for a block device
            VirtIODeviceType::BlockDevice => {
                let accepted_features = device_driver.init_driver(!(1 << 5))?;
                let mut block_driver = super::drivers::block::BlockDriver::new(device_driver);
                block_driver.device_specific(accepted_features)?;

                self.block_devices.push(Mutex::new(block_driver));
            }
            // Ignore any unknown devices
            VirtIODeviceType::UnknownDevice => {}
            default => {
                kwarnln!(unsafe "Unhandled device type {:?}", default);
            }
        }

        Ok(())
    }
}