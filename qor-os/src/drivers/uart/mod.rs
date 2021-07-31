//! Driver for a MMIO UART Interface

use crate::*;

use super::generic::ByteInterface;
use super::mmio;
use super::generic;

/// Safety: if the base address is a vaild base address for a UART driver,
/// this will perform as expected.
unsafe fn init(base: usize)
{
    // Set word length 0b11 will set an 8 bit word length
    let lcr = 0b0000011;
    mmio::write_offset::<u8>(base, 3, lcr);

    // Enable the recieve buffer interrupts
    mmio::write_offset::<u8>(base, 1, 0b0000001);

    // Divisor calculation
    let divisor = 592u16;
    let divisor_low = divisor & 0xFF;
    let divisor_high = (divisor & 0xFF00) >> 8;

    // Open the divisor latch
    mmio::write_offset::<u8>(base, 3, lcr | 1 << 7);

    mmio::write_offset::<u8>(base, 0, divisor_low as u8);
    mmio::write_offset::<u8>(base, 1, divisor_high as u8);

    // Close the divisor latch
    mmio::write_offset::<u8>(base, 3, lcr);
}

/// Read a byte from the UART port
/// Safety: if the base address is a vaild base address for an initialized UART
/// driver, this will perform as expected.
unsafe fn read_byte(base: usize) -> Option<u8>
{
    // Check if there is pending data
    if mmio::read_offset::<u8>(base, 5) & 1 == 0
    {
        None
    }
    else
    {
        Some(mmio::read_offset::<u8>(base, 0))
    }
}

/// Write a byte to the UART port
/// Safety: if the base address is a vaild base address for an initialized UART
/// driver, this will perform as expected.
unsafe fn write_byte(base: usize, data: u8)
{
    mmio::write_offset::<u8>(base, 0, data);
}

/// MMIO UART Driver
pub struct UARTDriver
{
    base: usize,
    buffer: utils::ByteRingBuffer
}

impl UARTDriver
{
    /// Create a new UART Driver
    /// Safety: if the base address is a vaild base address for a UART driver,
    /// this will perform as expected.
    pub const unsafe fn new(base: usize) -> Self
    {
        Self
        {
            base,
            buffer: utils::ByteRingBuffer::new()
        }
    }

    /// Initialize the UART Driver
    pub fn init(&mut self)
    {
        // Safety: Assuming the safety from the `new` implementation is
        // satisfied, this is safe
        unsafe 
        {
            init(self.base);
        }
    }

    /// Notify of a byte being recieved by the device
    pub fn notify_recieve(&mut self)
    {
        // Safety: Assuming the safety from the `new` implementation is
        // satisfied, this is safe
        if let Some(byte) = unsafe { read_byte(self.base) }
        {
            self.buffer.enqueue_byte(byte);
        }
    }
}

impl generic::ByteInterface for UARTDriver
{
    /// Read a byte from the UART
    fn read_byte(&mut self) -> Option<u8>
    {
        // self.buffer.dequeue_byte()

        unsafe { read_byte(self.base) }
    }

    /// Write a byte to the UART
    fn write_byte(&mut self, data: u8)
    {
        // Safety: Assuming the safety from the `new` implementation is
        // satisfied, this is safe
        unsafe 
        {
            write_byte(self.base, data);
        }   
    }
}

// Implement the core::fmt::Write trait for the UART Driver
impl core::fmt::Write for UARTDriver
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result
    {
        for byte in s.as_bytes()
        {
            self.write_byte(*byte);
        }

        Ok(())
    }
}