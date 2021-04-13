/// Driver for the hardware timer
pub struct TimerDriver
{
    base: usize
}

impl TimerDriver
{
    /// Create a new Timer Driver
    /// Safety: The base address must be the base address of a CLINT Timer
    pub unsafe fn new(base: usize) -> Self
    {
        Self
        {
            base
        }
    }

    /// Get the current time in microseconds since system start
    pub fn get_current_time(&self) -> usize
    {
        unsafe { super::ops::get_time(self.base) }
    }

    /// Set the remaining time in microseconds before the next interrupt
    pub fn set_remaining_time(&self, time: usize)
    {
        unsafe { super::ops::set_timer(self.base, time) }
    }
}