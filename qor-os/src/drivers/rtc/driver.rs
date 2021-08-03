use crate::*;

use fs::ioctl::IOControlCommand;

/// Real Time Clock Driver for the Goldfish RTC
pub struct RealTimeClockDriver
{
    base: usize
}

impl RealTimeClockDriver
{
    /// Create a new driver object
    /// Safety: The given base address must be a valid base address of a
    /// Goldfish RTC
    pub const unsafe fn new(base: usize) -> Self
    {
        Self
        {
            base
        }
    }

    /// Get the real time clock driver for the system
    pub const fn get_driver() -> Self
    {
        // Safety: This is the address given in the qemu spec
        unsafe { Self::new(0x101000) }
    }

    /// Get the current time in nanoseconds since Jan 1 1970
    pub fn get_unix_timestamp_nano(&self) -> u64
    {
        // See the safety requirements of the constructor
        unsafe { drivers::mmio::read_offset(self.base, 0) }
    }

    /// Handle an ioctl call
    pub fn exec_ioctl(&self, cmd: IOControlCommand) -> usize
    {
        match cmd
        {
            IOControlCommand::RealTimeClockGetTime { response } =>
            {
                let unix_nano = self.get_unix_timestamp_nano();

                *response = super::RTCTime::unix_timestamp_to_rtc_time(unix_nano / 1_000_000_000);

                0
            },

            _ => usize::MAX
        }
    }
}