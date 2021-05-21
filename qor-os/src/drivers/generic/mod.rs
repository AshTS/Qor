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
