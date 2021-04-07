// Macros
#[macro_export]
macro_rules! kprint
{
    ($($args:tt)+) => ({
        use core::fmt::Write;
		let _ = write!(crate::drivers::UART_DRIVER.lock(), $($args)+);    
    });
}

#[macro_export]
macro_rules! kprintln
{
    () => ({kprint!("\r\n")});

    ($fmt:expr) => ({
        kprint!(concat!($fmt, "\r\n"))
    });

    ($fmt:expr, $($args:tt)+) => ({
        kprint!(concat!($fmt, "\r\n"), $($args)+)
    });
}
