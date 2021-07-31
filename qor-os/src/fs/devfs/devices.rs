use crate::*;

use process::descriptor::*;

pub type DeviceFile = (&'static str, Box<dyn Fn() -> Box<dyn FileDescriptor>>);

/// Return all available device files for the system
pub fn get_device_files() -> Vec<DeviceFile>
{
    let mut result: Vec<DeviceFile> = Vec::new();

    // /dev/disp : Text mode for the frame buffer
    result.push(("disp", 
        Box::new(
            || Box::new(
                ByteInterfaceDescriptor::new(drivers::gpu::get_global_graphics_driver())
            ))));

    // /dev/fb0 : Raw frame buffer access
    result.push(("fb0", 
        Box::new(
            || Box::new(
                BufferDescriptor::new(drivers::gpu::get_global_graphics_driver())
            ))));

    result
}