//! Generic traits for drivers to implement to allow connections between drivers

/// Byte Interface Trait
/// Allows for the reading and writing of bytes to and from the given interface
pub trait ByteInterface
{
    /// Read a byte from the interface
    fn read_byte(&mut self) -> Option<u8>;

    /// Write a byte to the interface
    fn write_byte(&mut self, data: u8);
}

/// Block Device Driver Trait
/// Allows for reading and writing blocks from a block device
pub trait BlockDeviceDriver
{
    /// Read data from the block device
    fn read_data(buffer: *mut u8, offset: usize, size: usize);

    /// Write data to the block device
    fn write_data(buffer: *mut u8, offset: usize, size: usize);
}