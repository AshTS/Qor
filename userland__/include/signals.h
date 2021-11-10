#ifndef _SIGNALS_H
#define _SIGNALS_H

#include "types.h"

typedef unsigned long sigset_t;

union sigval
{
    int     sival_int;    /* Integer value */
    void   *sival_ptr;    /* Pointer value */
};

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
};

#define SIGINT 2
#define SIGKILL 9
#define SIGTERM 15

#define SIG_DFL 1
#define SIG_IGN 2

#define SA_SIGINFO 1

#endif _SIGNALS_H