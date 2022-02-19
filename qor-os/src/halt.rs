use crate::*;

pub fn kernel_halt()
{
    kprintln!("System Halt");

    unsafe { crate::drivers::POWER_DRIVER.shutdown() };
}