//! Kernel print and kernal print line functionss

/// Kernel print function
#[macro_export]
macro_rules! kprint
{
    ($($args:tt)+) => ({
        use core::fmt::Write;
		let _ = write!(crate::drivers::UART_DRIVER.lock(), $($args)+);    
    });
}

/// Kernel print line function
#[macro_export]
macro_rules! kprintln
{
    () => ({kprint!("\r\n")});

    ($fmt:expr) => ({
        crate::kprint!(concat!($fmt, "\r\n"))
    });

    ($fmt:expr, $($args:tt)+) => ({
        crate::kprint!(concat!($fmt, "\r\n"), $($args)+)
    });
}
