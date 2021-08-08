use super::super::PID;

/// Generic data structure for holding information about a signal
#[derive(Clone, Copy)]
pub union SignalValue
{
    pub integer: u32,
    pub ptr: usize
}

/// Signal Info Structure
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SignalInfo
{
    pub signal_number: u32,
    pub error: u32,
    pub code: u32,
    pub trap: u32,

    pub pid: PID,
    pub uid: u16,

    pub status: u32,
    pub utime: u64,
    pub stime: u64,

    pub value: SignalValue,
}

/*
struct siginfo_t
{
    int      si_signo;     /* Signal number */
    int      si_errno;     /* An errno value */
    int      si_code;      /* Signal code */
    int      si_trapno;    /* Trap number that caused
                                hardware-generated signal
                                (unused on most architectures) */
    pid_t    si_pid;       /* Sending process ID */
    uid_t    si_uid;       /* Real user ID of sending process */
    int      si_status;    /* Exit value or signal */
    clock_t  si_utime;     /* User time consumed */
    clock_t  si_stime;     /* System time consumed */
    union sigval si_value; /* Signal value */
};

struct sigaction
{
    void     (*sa_handler)(int);
    void     (*sa_sigaction)(int, struct siginfo_t *, void *);
    sigset_t   sa_mask;
    int        sa_flags;
    void     (*sa_restorer)(void);
};*/

/// Signal Action Structure
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignalAction
{
    pub handler_value: usize,
    pub action_fn_ptr: usize,
    pub mask: u64,
    pub flags: u32,
    pub restoring_addr: usize
}