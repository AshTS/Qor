/// Driver for the PLIC device
pub struct PLICDriver
{
    base: usize
}

impl PLICDriver
{
    /// Create a new PLIC Driver
    /// Safety: The base address must be the base address of a PLIC Device
    pub unsafe fn new(base: usize) -> Self
    {
        Self
        {
            base
        }
    }

    /// Enable the given interrupt id
    pub fn enable_interrupt(&self, id: u8)
    {
        unsafe { super::ops::enable_interrupt(self.base, id) }
    }

    /// Set the priority for a given interrupt
    pub fn set_priority(&self, id: u8, priority: u8)
    {
        unsafe { super::ops::set_priority(self.base, id, priority) }
    }

    /// Set the threshold
    pub fn set_threshold(&self, threshold: u8)
    {
        unsafe { super::ops::set_threshold(self.base, threshold) }
    }

    /// Get the next interrupt
    pub fn next(&self) -> Option<u32>
    {
        unsafe { super::ops::next(self.base) }
    }

    /// Complete the given interrupt
    pub fn complete(&self, id: u32)
    {
        unsafe { super::ops::complete(self.base, id) }
    }
}