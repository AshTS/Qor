//! Kernel debug print and debug print line functions

/// Kernel print function
#[macro_export]
macro_rules! kdebug
{
    ($($args:tt)+) => ({
        use core::fmt::Write;
		let _ = write!(crate::drivers::UART_DRIVER.lock(), $($args)+);   
        kprint!("\x1B[34m{}\x1B[m") 
    });
}

/// Kernel print line function
#[macro_export]
macro_rules! kdebugln
{
    () => ({kdebug!("\r\n")});

    ($fmt:expr) => ({
        crate::kdebug!(concat!($fmt, "\r\n"))
    });

    ($fmt:expr, $($args:tt)+) => ({
        crate::kdebug!(concat!($fmt, "\r\n"), $($args)+)
    });
}
