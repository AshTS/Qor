use crate::*;

pub fn kernel_halt() {
    kprintln!(unsafe "System Halt");
    crate::drivers::POWER_DRIVER.shutdown_success();
}
