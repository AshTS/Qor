//! Generic traits for drivers to implement to allow connections between drivers

/// Byte Interface Trait
/// Allows for the reading and writing of bytes to and from the given interface
pub trait ByteInterface
{
    /// Read a byte from the interface
    fn read_byte(&mut self) -> Option<u8>;

    /// Write a byte to the interface
    fn write_byte(&mut self, data: u8);

    /// Flush the interface
    fn flush(&mut self) {}
}

/// Buffer Interface Trait
/// Allows reading and writing to and from a buffer in memory
pub trait BufferInterface
{
    /// Read a byte
    fn read_byte(&mut self, offset: usize) -> Option<u8>;

    /// Write a byte
    fn write_byte(&mut self, offset: usize, data: u8);

    /// Get the size of the buffer
    fn get_size(&self) -> usize;

    /// Flush the memory (send an update to wherever it is pointing)
    fn flush(&mut self);
}