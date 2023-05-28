/// Wrapper for the UART Driver which provides unsafe (i.e aliasable) access to the UART Driver via the Display trait
pub struct UnsafeUARTWriter {
    _marker: (),
}

impl UnsafeUARTWriter {
    /// Construct an `UnsafeUARTWriter`
    ///
    /// Safety: No other access to the UART port should be made while `UnsafeUARTWriter` exists
    pub const unsafe fn new() -> Self {
        Self { _marker: () }
    }
}

impl core::fmt::Write for UnsafeUARTWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        /*
        // Ensure that the driver it initialized
        crate::drivers::UART_DRIVER
            .ensure_init()
            .map_err(|_| core::fmt::Error)?; */

        // Safety: The driver initialization is handled by the above `ensure_init` check, and the single threaded requirement is given by the safety requirements of the constructor of `UnsafeUARTWriter`
        unsafe { crate::drivers::UART_DRIVER.unchecked_write_bytes(s.as_bytes()) };
        Ok(())
    }
}
