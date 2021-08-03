use crate::*;

use fs::ioctl::IOControlCommand;

use process::descriptor::*;

use fs::structures::FilesystemIndex;

/// DeviceFile object
pub struct DeviceFile
{
    pub name: &'static str,
    desc_const: Box<dyn Fn(FilesystemIndex) -> Box<dyn FileDescriptor>>,
    io_ctl: Box<dyn Fn(IOControlCommand) -> usize>
}

impl DeviceFile
{
    /// Create a new device file
    pub fn new(name: &'static str, desc_const: Box<dyn Fn(FilesystemIndex) -> Box<dyn FileDescriptor>>,
               io_ctl: Box<dyn Fn(IOControlCommand) -> usize>) -> Self
    {
        Self
        {
            name, desc_const, io_ctl
        }
    }

    /// Make the descriptor
    pub fn make_descriptor(&self, index: FilesystemIndex) -> Box<dyn FileDescriptor>
    {
        (self.desc_const)(index)
    }

    /// Execute an ioctl command on the driver
    pub fn exec_ioctl(&self, cmd: IOControlCommand) -> usize
    {
        (self.io_ctl)(cmd)
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
                    |inode| Box::new(
                        ByteInterfaceDescriptor::new(drivers::gpu::get_global_graphics_driver(), inode)
                    )),
                    Box::new( |_| usize::MAX)
                ));

        // /dev/fb0 : Raw frame buffer access
        result.push(
            DeviceFile::new(
                "fb0",
                Box::new(
                    |inode| Box::new(
                        BufferDescriptor::new(drivers::gpu::get_global_graphics_driver(), inode)
                    )),
                    Box::new( |cmd| drivers::gpu::get_global_graphics_driver().exec_ioctl(cmd))
                ));
    }

    // /dev/tty0 : UART Port
    result.push(
        DeviceFile::new(
            "tty0",
            Box::new(
                |inode| Box::new(
                    ByteInterfaceDescriptor::new(drivers::get_uart_driver(), inode)
                )),
                Box::new( |_| usize::MAX)
            ));

    // /dev/null : Null Descriptor
    result.push(
        DeviceFile::new(
            "null",
            Box::new(
                |inode| Box::new(
                    NullDescriptor{ inode })
                ),
            Box::new( |_| usize::MAX)
        ));

    result
}