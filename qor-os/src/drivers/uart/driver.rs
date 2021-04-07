use super::ops;

/// UART NS16550a Memory Mapped IO Driver
pub struct UartDriver
{
    base_address: usize,
}

impl UartDriver
{
    /// Create a new UartDriver object with the given base address
    ///
    /// Safety: If the base address is the base address of the Memory Mapped IO
    /// Interface to the UART NS16550a Chipset, the struct will function as
    /// expected.
    pub unsafe fn new(base_address: usize) -> UartDriver
    {
        ops::uart_init(base_address);

        Self
        {
            base_address
        }
    }

    /// Read a byte from the UART (or return None if no byte is available)
    pub fn read_byte(&self) -> Option<u8>
    {
        // Safety: Assuming the struct was initialized properly, this will be
        // making use of a valid MMIO interface
        unsafe
        {
            ops::uart_read(self.base_address)
        }
    }

    /// Write a byte to the UART
    pub fn write_byte(&self, data: u8)
    {
        // Safety: Assuming the struct was initialized properly, this will be
        // making use of a valid MMIO interface
        unsafe
        {
            ops::uart_write(self.base_address, data);
        }
    }
}

impl core::fmt::Write for UartDriver
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result
    {
        for b in s.as_bytes()
        {
            self.write_byte(*b);
        }

        Ok(())
    }
}