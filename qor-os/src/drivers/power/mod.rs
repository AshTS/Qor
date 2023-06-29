/// Simple dummy power driver which allows us to terminate the virtual machine instance
pub struct PowerDriver
{
    base: usize
}

impl PowerDriver
{
    /// Constructs a new PowerDriver instance at the given base address
    /// Safety: The given base address must be a valid base address for the power device
    pub const unsafe fn new(base: usize) -> Self
    {
        Self
        {
            base
        }
    }

    /// Shutdown the virtual machine
    pub fn shutdown(&self)
    {
        unsafe
        {
            (self.base as *mut u32).write_volatile(0x5555);
        }
    }
}