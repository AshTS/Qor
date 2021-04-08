//! Kernel debug print and debug print line functions

/// Kernel print function
#[macro_export]
macro_rules! kdebug
{
    ($fmt:expr, $($args:tt)*) => ({
        kprint!(concat!("\x1B[34m", $fmt, "\x1B[m"), $($args)*) 
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
