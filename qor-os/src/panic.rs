//! Panic Implementation

use crate::*;

/// Panic handler for the kernel
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    kerror!(unsafe "\nAborting: ");

    if let Some(p) = info.location() {
        kerrorln!(unsafe "line {}, file {}: {}", p.line(), p.file(), info.message().unwrap());
    } else {
        kerrorln!(unsafe "no info available");
    }

    #[cfg(test)]
    unsafe { crate::drivers::POWER_DRIVER.shutdown_failure() };

    abort();
}

/// Terminate execution by waiting in a loop
#[no_mangle]
extern "C" fn abort() -> ! {
    loop {
        unsafe { riscv::asm::wfi() };
    }
}
