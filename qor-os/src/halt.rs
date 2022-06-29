use crate::*;

pub fn kernel_halt()
{
    kprintln!(unsafe "System Halt");

    // unsafe { crate::drivers::POWER_DRIVER.shutdown() };
}