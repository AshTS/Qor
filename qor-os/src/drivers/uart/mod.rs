mod errors;
mod raw;
mod writer;

use libutils::sync::InitThreadMarker;
pub use errors::UARTDriverError;
use writer::UnsafeUARTWriter;

/// Memory Mapped UART Driver
/// 
/// The driver is intended to be used either during kernel boot (to give a single threaded output) through passing an 'InitThreadMarker' object to the helper functions.
pub struct UARTDriver
{
    base_address: usize,
    is_initialized: core::sync::atomic::AtomicBool
}

impl UARTDriver
{
    /// Statically construct the UART Driver from the `base_address`
    /// 
    /// Note that this does not actually initialize the UART Driver, that function needs to be called seperately
    /// 
    /// Safety: The base address given must be a base address for an MMIO UART driver
    pub const unsafe fn new(base_address: usize) -> Self
    {
        Self
        {
            base_address,
            is_initialized: core::sync::atomic::AtomicBool::new(false)
        }
    }
    
    /// Ensure that the UART Driver has been initialized
    fn ensure_init(&self) -> Result<(), UARTDriverError>
    {
        if self.is_initialized.load(core::sync::atomic::Ordering::SeqCst)
        {
            Ok(())
        }   
        else
        {
            Err(UARTDriverError::UartNotInitialized)
        }
    }

    /// Initialize the UART Driver
    pub fn init(&self, _marker: InitThreadMarker)
    {
        // Safety: The base address must be valid per the safety requirements of the `UARTDriver` constructor, and there are no other threads as we have ownership of an `InitThreadMarker`
        unsafe { raw::init(self.base_address) }

        self.is_initialized.store(true, core::sync::atomic::Ordering::SeqCst);
    }

    /// Write a byte to the UART port without checking single threadedness and initialization
    /// 
    /// Safety: This function requires that only a single thread is accessing the UART port and that the UART port has been initialized
    pub unsafe fn unchecked_write_byte(&self, byte: u8)
    {
        // Safety: The base address must be valid per the safety requirements of the `UARTDriver` constructor, the initialization and single threaded requirements are given by this function's safety requirements. 
        raw::write_byte(self.base_address, byte)
    }

    /// Read a byte from the UART port without checking single threadedness and initialization
    /// 
    /// Safety: This function requires that only a single thread is accessing the UART port and that the UART port has been initialized
    pub unsafe fn unchecked_read_byte(&self) -> Option<u8>
    {
        // Safety: The base address must be valid per the safety requirements of the `UARTDriver` constructor, the initialization and single threaded requirements are given by this function's safety requirements.
        raw::read_byte(self.base_address)
    }

    /// Write a byte to the UART port
    pub fn write_byte(&self, _marker: InitThreadMarker, byte: u8) -> Result<(), UARTDriverError>
    {
        // Ensure initialization requirement
        self.ensure_init()?;

        // Safety: The initialization requirement is given by the `ensure_init` call, and the single threaded requirement is given by the `InitThreadMarker`
        unsafe { self.unchecked_write_byte(byte); }

        Ok(())
    }

    /// Read a byte from the UART port
    pub fn read_byte(&self, _marker: InitThreadMarker) -> Result<Option<u8>, UARTDriverError>
    {
        // Ensure initialization requirement
        self.ensure_init()?;

        // Safety: The initialization requirement is given by the `ensure_init` call, and the single threaded requirement is given by the `InitThreadMarker`
        unsafe { Ok(self.unchecked_read_byte()) }
    }

    /// Write a slice of bytes to the UART port
    pub fn write_bytes(&self, marker: InitThreadMarker, bytes: &[u8]) -> Result<(), UARTDriverError>
    {
        for byte in bytes
        {
            self.write_byte(marker, *byte)?;
        }

        Ok(())
    }

    /// Write a slice of bytes to the UART port without checking single threadedness and initialization
    /// 
    /// Safety: This function requires that only a single thread is accessing the UART port and that the UART port has been initialized
    pub unsafe fn unchecked_write_bytes(&self, bytes: &[u8])
    {
        for byte in bytes
        {
            // Safety: The base address must be valid per the safety requirements of the `UARTDriver` constructor, the initialization and single threaded requirements are given by this function's safety requirements. 
            self.unchecked_write_byte(*byte);
        }
    }

    /// Get an `UnsafeUartWriter` from the `UARTDriver`
    /// 
    /// Safety: No other access to the `UARTDriver` should be made while the `UnsafeUARTWriter` lives
    pub unsafe fn unsafe_writer(&self) -> UnsafeUARTWriter
    {
        // Safety: The single threaded requirement is given by the safety requirements of this function
        UnsafeUARTWriter::new()
    }
}
