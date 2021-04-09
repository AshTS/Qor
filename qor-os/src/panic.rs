use crate::*;

/// Panic handler for the kernel
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> !
{
    kprint!("\x1B[31mAborting: ");

    if let Some(p) = info.location()
    {
        kprintln!("line {}, file {}: {}", p.line(), p.file(), info.message().unwrap());
    }
    else
    {
        kprintln!("no info available");
    }

    kprint!("\x1B[m");

    abort();
}

/// Terminate execution by waiting in a loop
#[no_mangle]
extern "C"
fn abort() -> !
{
    loop
    {
        unsafe{asm!("wfi")}
    }
}