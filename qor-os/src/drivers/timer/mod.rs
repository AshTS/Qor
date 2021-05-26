// Frequency of the timer
const FREQUENCY: usize = 10_000_000; 

/// Structure to store a time value
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct KernelTime(usize);

impl KernelTime
{
    /// Create a new Kernel Timing value
    pub fn new(time: usize) -> Self
    {
        Self(time)
    }

    /// Create a new Kernel Timing value from seconds
    pub fn seconds(seconds: usize) -> Self
    {
        Self(seconds * FREQUENCY)
    }

    /// Create a new Kernel Timing value from milliseconds
    pub fn milliseconds(milliseconds: usize) -> Self
    {
        Self(milliseconds * FREQUENCY / 1_000)
    }

    /// Create a new Kernel Timing value from microseconds
    pub fn microseconds(microseconds: usize) -> Self
    {
        Self(microseconds * FREQUENCY / 1_000_000)
    }
}

/// Timer Driver
pub struct TimerDriver
{
    base: usize,
    interval: KernelTime
}

impl TimerDriver
{
    /// Create a new timer driver
    /// Safety: The base address given must be a valid base address for an mmio timer
    pub const unsafe fn new(base: usize) -> Self
    {
        Self
        {
            base,
            interval: KernelTime(0)
        }
    }

    /// Get the current time
    pub fn time(&self) -> KernelTime
    {
        // Safety: Assuming the base is a valid base address (as is the case for
        // the initialization requirements), this is safe
        unsafe 
        {
            KernelTime(crate::drivers::mmio::read_offset::<u64>(self.base, 0xBFF8) as usize)
        }
    }

    /// Set the remaining time
    pub fn set_remaining(&mut self, remaining: KernelTime)
    {
        // Safety: Assuming the base is a valid base address (as is the case for
        // the initialization requirements), this is safe
        unsafe 
        {
            crate::drivers::mmio::write_offset::<u64>(self.base, 0x4000, (self.time().0 + remaining.0) as u64);
        }
    }

    /// Reset the timer
    pub fn reset_timer(&mut self)
    {
        // Safety: Assuming the base is a valid base address (as is the case for
        // the initialization requirements), this is safe
        unsafe 
        {
            crate::drivers::mmio::write_offset::<u64>(self.base, 0xBFF8, 0);
        }
    }

    /// Triggered when the timer interrupt is struck
    pub fn trigger(&mut self)
    {
        self.set_remaining(self.interval)
    }

    /// Set the interval for the timer
    pub fn set_interval(&mut self, interval: KernelTime)
    {
        self.interval = interval;
    }

    /// Set the frequency of the timer
    pub fn set_frequency(&mut self, frequency: usize)
    {
        self.set_interval(KernelTime(FREQUENCY / frequency))
    }
}