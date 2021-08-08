use super::structs::*;

use super::super::PID;

/// POSIX Signal Types
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SignalType
{
    SIGTRAP = 5,
    SIGTERM = 15,
    SIGSTOP = 19,
    SIGCONT = 18,
    SIGKILL = 9,
    SIGINT = 2
}

impl SignalType
{
    /// Convert a number to a signal type
    pub fn number_to_signal(num: usize) -> Self
    {
        match num
        {
            2 => Self::SIGINT,
            5 => Self::SIGTRAP,
            9 => Self::SIGKILL,
            15 => Self::SIGTERM,
            18 => Self::SIGCONT,
            19 => Self::SIGSTOP,
            default => panic!("Bad signal number {}", default)
        }
    }
}

/// POSIX Signal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct POSIXSignal
{
    pub sig_type: SignalType,
    pub dest_pid: PID,
    pub sending_pid: PID
}

impl POSIXSignal
{
    /// Create a new signal object from the destination PID and the signal type
    pub fn new(dest_pid: PID, sending_pid: PID, sig_type: SignalType) -> Self
    {
        Self
        {
            sig_type,
            dest_pid,
            sending_pid,
        }
    }

    /// Convert to a SignalInfo structure for interacting with userland
    pub fn to_sig_info(&self) -> SignalInfo
    {
        SignalInfo
        {
            signal_number: self.sig_type as u16 as u32,
            error: 0,
            code: 0,
            trap: 0,
            pid: self.sending_pid,
            uid: 0,
            status: 0,
            utime: 0,
            stime: 0,
            value: SignalValue { integer: 0 },
        }
    }
}