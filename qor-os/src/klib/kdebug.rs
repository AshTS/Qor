//! Kernel debug print and debug print line functions

/// Kernel print function
#[macro_export]
macro_rules! kdebug
{
    ($mode:ident, $fmt:expr, $($args:tt)+) => ({
        if crate::debug::check_debug(crate::debug::DebugMode::$mode) { crate::kprint!(concat!("\x1B[34m", $fmt, "\x1B[m"), $($args)+) } 
    });
    
    ($mode:ident, $fmt:expr) => ({
        if crate::debug::check_debug(crate::debug::DebugMode::$mode) { crate::kprint!(concat!("\x1B[34m", $fmt, "\x1B[m")) }
    });

    ($fmt:expr, $($args:tt)+) => ({
        crate::kdebug!(Other, $fmt, $($args)+) 
    });
    
    ($fmt:expr) => ({
        crate::kprint!(Other, $fmt) 
    });
}

/// Kernel print line function
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
