// Flag to set if the output should be colored
pub const COLORED: bool = true;

/// Kernel print function
#[macro_export]
macro_rules! kprint
{
    ($($args:tt)+) => ({
        use core::fmt::Write;

        // Safety: This is safe because overlapping writes is acceptable if annoying
		let _ = write!(unsafe { &mut crate::drivers::UART_DRIVER }, $($args)+);    
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

/// Kernel debug print function
#[macro_export]
macro_rules! kdebug
{
    ($mode:ident, $fmt:expr, $($args:tt)+) => ({
        if crate::debug::check_debug(crate::debug::DebugCategories::$mode)
        {
            if crate::kprint::COLORED
            {
                crate::kprint!(concat!("\x1B[34m", $fmt, "\x1B[1m"), $($args)+)
            }
            else
            {
                crate::kprint!($fmt, $($args)+);
            }
        }
    });
    
    ($mode:ident, $fmt:expr) => ({
        if crate::debug::check_debug(crate::debug::DebugCategories::$mode)
        {
            if crate::kprint::COLORED
            {
                crate::kprint!(concat!("\x1B[34m", $fmt, "\x1B[1m"))
            }
            else
            {
                crate::kprint!($fmt);
            }
        }
    });

    ($fmt:expr, $($args:tt)+) => ({
        crate::kdebug!(Other, $fmt, $($args)+) 
    });
    
    ($fmt:expr) => ({
        crate::kdebug!(Other, $fmt) 
    });
}

/// Kernel debug print line function
#[macro_export]
macro_rules! kdebugln
{
    ($mode:ident) => ({kdebug!($mode, "\r\n")});

    ($mode:ident, $fmt:expr) => ({
        crate::kdebug!($mode, concat!($fmt, "\r\n"))
    });

    ($mode:ident, $fmt:expr, $($args:tt)+) => ({
        crate::kdebug!($mode, concat!($fmt, "\r\n"), $($args)+)
    });

    () => ({kdebug!(Other, "\r\n")});

    ($fmt:expr) => ({
        crate::kdebug!(Other, concat!($fmt, "\r\n"))
    });

    ($fmt:expr, $($args:tt)+) => ({
        crate::kdebug!(Other, concat!($fmt, "\r\n"), $($args)+)
    });
}

/// Kernel warn print function
#[macro_export]
macro_rules! kwarn
{
    ($fmt:expr, $($args:tt)+) => ({
        if crate::kprint::COLORED
        {
            crate::kprint!(concat!("\x1B[33m", $fmt, "\x1B[1m"), $($args)+)
        }
        else
        {
            crate::kprint!($fmt, $($args)+);
        }
    });
    
    ($fmt:expr) => ({
        if crate::kprint::COLORED
        {
            crate::kprint!(concat!("\x1B[33m", $fmt, "\x1B[1m"))
        }
        else
        {
            crate::kprint!($fmt);
        }
    });
}

/// Kernel warn print line function
#[macro_export]
macro_rules! kwarnln
{
    () => ({crate::kwarn!("\r\n")});

    ($fmt:expr) => ({
        crate::kwarn!(concat!($fmt, "\r\n"))
    });

    ($fmt:expr, $($args:tt)+) => ({
        crate::kwarn!(concat!($fmt, "\r\n"), $($args)+)
    });
}

/// Kernel error print function
#[macro_export]
macro_rules! kerror
{
    ($fmt:expr, $($args:tt)+) => ({
        if crate::kprint::COLORED
        {
            crate::kprint!(concat!("\x1B[31m", $fmt, "\x1B[1m"), $($args)+)
        }
        else
        {
            crate::kprint!($fmt, $($args)+);
        }
    });
    
    ($fmt:expr) => ({
        if crate::kprint::COLORED
        {
            crate::kprint!(concat!("\x1B[31m", $fmt, "\x1B[1m"))
        }
        else
        {
            crate::kprint!($fmt);
        }
    });
}

/// Kernel error print line function
#[macro_export]
macro_rules! kerrorln
{
    () => ({crate::kerror!("\r\n")});

    ($fmt:expr) => ({
        crate::kerror!(concat!($fmt, "\r\n"))
    });

    ($fmt:expr, $($args:tt)+) => ({
        crate::kerror!(concat!($fmt, "\r\n"), $($args)+)
    });
}