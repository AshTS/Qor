use super::SignalDisposition;

/// POSIX Signal Types
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalType
{
    SIGTRAP,
    SIGTERM,
    SIGSTOP,
    SIGCONT,
}

/// POSIX Signal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct POSIXSignal
{
    pub sig_type: SignalType,
    pub disposition: SignalDisposition,
    pub dest_pid: u16
}

impl POSIXSignal
{
    /// Create a new signal object from the destination PID and the signal type
    pub fn new(dest_pid: u16, sig_type: SignalType) -> Self
    {
        Self
        {
            sig_type,
            dest_pid,
            disposition: match sig_type
            {
                SignalType::SIGTRAP => SignalDisposition::Core,
                SignalType::SIGTERM => SignalDisposition::Terminate,
                SignalType::SIGSTOP => SignalDisposition::Stop,
                SignalType::SIGCONT => SignalDisposition::Continue,
            }
        }
    }
}