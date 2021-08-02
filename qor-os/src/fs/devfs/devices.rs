use crate::*;

use process::descriptor::*;

use fs::structures::FilesystemIndex;

/// DeviceFile object
pub struct DeviceFile
{
    pub name: &'static str,
    pub desc_const: Box<dyn Fn(FilesystemIndex) -> Box<dyn FileDescriptor>>
}

impl DeviceFile
{
    /// Create a new device file
    pub fn new(name: &'static str, desc_const: Box<dyn Fn(FilesystemIndex) -> Box<dyn FileDescriptor>>) -> Self
    {
        Self
        {
            name, desc_const
        }
    }

    /// Make the descriptor
    pub fn make_descriptor(&self, index: FilesystemIndex) -> Box<dyn FileDescriptor>
    {
        (self.desc_const)(index)
    }
}

/// Return all available device files for the system
pub fn get_device_files() -> Vec<DeviceFile>
{
    let mut result: Vec<DeviceFile> = Vec::new();
    
    // Only add graphics devices if the graphics driver is loaded
    if drivers::gpu::is_graphics_driver_loaded()
    {
        // /dev/disp : Text mode for the frame buffer
        result.push(
            DeviceFile::new(
                "disp",
                Box::new(
                    |_index| Box::new(
                        ByteInterfaceDescriptor::new(drivers::gpu::get_global_graphics_driver())
                    ))));

        // /dev/fb0 : Raw frame buffer access
        result.push(
            DeviceFile::new(
                "fb0",
                Box::new(
                    |_index| Box::new(
                        BufferDescriptor::new(drivers::gpu::get_global_graphics_driver())
                    ))));
    }

    // /dev/tty0 : UART Port
    result.push(
        DeviceFile::new(
            "tty0",
            Box::new(
                |_index| Box::new(
                    ByteInterfaceDescriptor::new(drivers::get_uart_driver())
                ))));

    // /dev/null : Null Descriptor
    result.push(
        DeviceFile::new(
            "null",
            Box::new(
                |_index| Box::new(
                    NullDescriptor{})
                )));

    result
}