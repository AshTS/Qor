// Flag to set if the output should be colored
pub const COLORED: bool = true;

/// Kernel print function
#[macro_export]
macro_rules! kprint
{
    (unsafe $($args:tt)+) => ({
        use core::fmt::Write;

        // Safety: This is safe because overlapping writes is acceptable if annoying
        #[allow(unused_unsafe)]
		let _ = write!(unsafe { crate::drivers::UART_DRIVER.unsafe_writer() }, $($args)+);    
    });

    ($thread_marker:expr, $($args:tt)+) => ({
        use core::fmt::Write;
        use libutils::sync::InitThreadMarker;

        // Safety: This is safe because overlapping writes is acceptable if annoying
		let _ = (|_: InitThreadMarker| { write!(unsafe { crate::drivers::UART_DRIVER.unsafe_writer() }, $($args)+) })($thread_marker);    
    });
}

/// Kernel print line function
#[macro_export]
macro_rules! kprintln
{
    (unsafe) => ({kprint!("\r\n")});

    (unsafe $fmt:expr) => ({
        crate::kprint!(unsafe concat!($fmt, "\r\n"))
    });

    (unsafe $fmt:expr, $($args:tt)+) => ({
        crate::kprint!(unsafe concat!($fmt, "\r\n"), $($args)+)
    });

    ($thread_marker:expr) => ({kprint!($thread_marker, "\r\n")});

    ($thread_marker:expr, $fmt:expr) => ({
        crate::kprint!($thread_marker, concat!($fmt, "\r\n"))
    });

    ($thread_marker:expr, $fmt:expr, $($args:tt)+) => ({
        crate::kprint!($thread_marker, concat!($fmt, "\r\n"), $($args)+)
    });
}

/// Kernel debug print function
#[macro_export]
macro_rules! kdebug
{
    (unsafe $mode:ident, $fmt:expr, $($args:tt)+) => ({
        if crate::debug::check_debug(crate::debug::DebugCategories::$mode)
        {
            if crate::kprint::COLORED
            {
                crate::kprint!(unsafe concat!("\x1B[34m", $fmt, "\x1B[m"), $($args)+)
            }
            else
            {
                crate::kprint!(unsafe $fmt, $($args)+);
            }
        }
    });
    
    (unsafe $mode:ident, $fmt:expr) => ({
        if crate::debug::check_debug(crate::debug::DebugCategories::$mode)
        {
            if crate::kprint::COLORED
            {
                crate::kprint!(unsafe concat!("\x1B[34m", $fmt, "\x1B[m"))
            }
            else
            {
                crate::kprint!(unsafe $fmt);
            }
        }
    });

    ($thread_marker:expr, $mode:ident, $fmt:expr, $($args:tt)+) => ({
        if crate::debug::check_debug(crate::debug::DebugCategories::$mode)
        {
            if crate::kprint::COLORED
            {
                crate::kprint!($thread_marker, concat!("\x1B[34m", $fmt, "\x1B[m"), $($args)+)
            }
            else
            {
                crate::kprint!($thread_marker, $fmt, $($args)+);
            }
        }
    });
    
    ($thread_marker:expr, $mode:ident, $fmt:expr) => ({
        if crate::debug::check_debug(crate::debug::DebugCategories::$mode)
        {
            if crate::kprint::COLORED
            {
                crate::kprint!($thread_marker, concat!("\x1B[34m", $fmt, "\x1B[m"))
            }
            else
            {
                crate::kprint!($thread_marker, $fmt);
            }
        }
    });

    (unsafe $fmt:expr, $($args:tt)+) => ({
        crate::kdebug!(unsafe Other, $fmt, $($args)+) 
    });
    
    (unsafe $fmt:expr) => ({
        crate::kdebug!(unsafe Other, $fmt) 
    });

    ($thread_marker:expr, $fmt:expr, $($args:tt)+) => ({
        crate::kdebug!($thread_marker, Other, $fmt, $($args)+) 
    });
    
    ($thread_marker:expr, $fmt:expr) => ({
        crate::kdebug!($thread_marker, Other, $fmt) 
    });
}

/// Kernel debug print line function
#[macro_export]
macro_rules! kdebugln
{
    (unsafe $mode:ident) => ({kdebug!(unsafe $mode, "\r\n")});

    (unsafe $mode:ident, $fmt:expr) => ({
        crate::kdebug!(unsafe $mode, concat!($fmt, "\r\n"))
    });

    (unsafe $mode:ident, $fmt:expr, $($args:tt)+) => ({
        crate::kdebug!(unsafe $mode, concat!($fmt, "\r\n"), $($args)+)
    });

    (unsafe) => ({kdebug!(unsafe Other, "\r\n")});

    (unsafe $fmt:expr) => ({
        crate::kdebug!(unsafe Other, concat!($fmt, "\r\n"))
    });

    (unsafe $fmt:expr, $($args:tt)+) => ({
        crate::kdebug!(unsafe Other, concat!($fmt, "\r\n"), $($args)+)
    });

    //
    ($thread_marker:expr, $mode:ident) => ({kdebug!($thread_marker, $mode, "\r\n")});

    ($thread_marker:expr, $mode:ident, $fmt:expr) => ({
        crate::kdebug!($thread_marker, $mode, concat!($fmt, "\r\n"))
    });

    ($thread_marker:expr, $mode:ident, $fmt:expr, $($args:tt)+) => ({
        crate::kdebug!($thread_marker, $mode, concat!($fmt, "\r\n"), $($args)+)
    });

    ($thread_marker:expr) => ({kdebug!($thread_marker, Other, "\r\n")});

    ($thread_marker:expr, $fmt:expr) => ({
        crate::kdebug!($thread_marker, Other, concat!($fmt, "\r\n"))
    });

    ($thread_marker:expr, $fmt:expr, $($args:tt)+) => ({
        crate::kdebug!($thread_marker, Other, concat!($fmt, "\r\n"), $($args)+)
    });
}

/// Kernel warn print function
#[macro_export]
macro_rules! kwarn
{
    (unsafe $fmt:expr, $($args:tt)+) => ({
        if crate::kprint::COLORED
        {
            crate::kprint!(unsafe concat!("\x1B[33m", $fmt, "\x1B[m"), $($args)+)
        }
        else
        {
            crate::kprint!(unsafe $fmt, $($args)+);
        }
    });
    
    (unsafe $fmt:expr) => ({
        if crate::kprint::COLORED
        {
            crate::kprint!(unsafe concat!("\x1B[33m", $fmt, "\x1B[m"))
        }
        else
        {
            crate::kprint!(unsafe $fmt);
        }
    });

    ($thread_marker:expr, $fmt:expr, $($args:tt)+) => ({
        if crate::kprint::COLORED
        {
            crate::kprint!(concat!("\x1B[33m", $fmt, "\x1B[m"), $($args)+)
        }
        else
        {
            crate::kprint!($fmt, $($args)+);
        }
    });
    
    ($thread_marker:expr, $fmt:expr) => ({
        if crate::kprint::COLORED
        {
            crate::kprint!($thread_marker, concat!("\x1B[33m", $fmt, "\x1B[m"))
        }
        else
        {
            crate::kprint!($thread_marker, $fmt);
        }
    });
}

/// Kernel warn print line function
#[macro_export]
macro_rules! kwarnln
{
    (unsafe) => ({crate::kwarn!(unsafe "\r\n")});

    (unsafe $fmt:expr) => ({
        crate::kwarn!(unsafe concat!($fmt, "\r\n"))
    });

    (unsafe $fmt:expr, $($args:tt)+) => ({
        crate::kwarn!(unsafe concat!($fmt, "\r\n"), $($args)+)
    });

    ($thread_marker:expr) => ({crate::kwarn!($thread_marker, "\r\n")});

    ($thread_marker:expr, $fmt:expr) => ({
        crate::kwarn!($thread_marker, concat!($fmt, "\r\n"))
    });

    ($thread_marker:expr, $fmt:expr, $($args:tt)+) => ({
        crate::kwarn!($thread_marker, concat!($fmt, "\r\n"), $($args)+)
    });
}

/// Kernel error print function
#[macro_export]
macro_rules! kerror
{
    (unsafe $fmt:expr, $($args:tt)+) => ({
        if crate::kprint::COLORED
        {
            crate::kprint!(unsafe concat!("\x1B[31m", $fmt, "\x1B[m"), $($args)+)
        }
        else
        {
            crate::kprint!(unsafe $fmt, $($args)+);
        }
    });
    
    (unsafe $fmt:expr) => ({
        if crate::kprint::COLORED
        {
            crate::kprint!(unsafe concat!("\x1B[31m", $fmt, "\x1B[m"))
        }
        else
        {
            crate::kprint!(unsafe $fmt);
        }
    });

    ($thread_marker:expr, $fmt:expr, $($args:tt)+) => ({
        if crate::kprint::COLORED
        {
            crate::kprint!(concat!("\x1B[31m", $fmt, "\x1B[m"), $($args)+)
        }
        else
        {
            crate::kprint!($fmt, $($args)+);
        }
    });
    
    ($thread_marker:expr, $fmt:expr) => ({
        if crate::kprint::COLORED
        {
            crate::kprint!($thread_marker, concat!("\x1B[31m", $fmt, "\x1B[m"))
        }
        else
        {
            crate::kprint!($thread_marker, $fmt);
        }
    });
}

/// Kernel error print line function
#[macro_export]
macro_rules! kerrorln
{
    (unsafe) => ({crate::kerror!(unsafe "\r\n")});

    (unsafe $fmt:expr) => ({
        crate::kerror!(unsafe concat!($fmt, "\r\n"))
    });

    (unsafe $fmt:expr, $($args:tt)+) => ({
        crate::kerror!(unsafe concat!($fmt, "\r\n"), $($args)+)
    });

    ($thread_marker:expr) => ({crate::kerror!($thread_marker, "\r\n")});

    ($thread_marker:expr, $fmt:expr) => ({
        crate::kerror!($thread_marker, concat!($fmt, "\r\n"))
    });

    ($thread_marker:expr, $fmt:expr, $($args:tt)+) => ({
        crate::kerror!($thread_marker, concat!($fmt, "\r\n"), $($args)+)
    });
}