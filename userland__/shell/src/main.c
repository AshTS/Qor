#include "printf.h"
#include "syscalls.h"
#include "string.h"
#include "signals.h"

#include <stdbool.h>

char* PATH = "/bin/";

static int RUNNING_PID = 0;
static bool WAITING = true;
static bool IS_TIMING = false;

void display_tag();

int handle_redirect(char** argv);

int run_exec(char* exec, char** argv, char** envp);
int run_exec_time(char* exec, char** argv, char** envp);

void handler(int sig, struct siginfo_t *info, void *ucontext)
{
    // printf("Got SIGINT\n");

    if (RUNNING_PID > 0)
    {
        kill(RUNNING_PID, SIGINT);
        printf("\n");
    }
    else
    {
        WAITING = false;
    }

    IS_TIMING = false;

    sigreturn();
}

int main()
{
    // Setup the handler for SIGINT
    struct sigaction new;
    struct sigaction old;

    new.sa_flags = SA_SIGINFO;
    new.sa_sigaction = handler;

    sigaction(SIGINT, &new, &old);

    char* envp[1];
    envp[0] = 0;

    char* argv[64];
    argv[0] = 0;

    while (1)
    {
        display_tag();

        WAITING = true;

        char buffer[64];
        while (WAITING)
        {
            int count = read(0, buffer, 63);

            if (count == 0) continue;

            buffer[count - 1] = 0;

            break;
        }

        if (buffer[0] == 0)
        {
            continue;
        }

        if (!WAITING)
        {
            printf("\n");
            continue;
        }

        if (buffer[0] == 'c' && buffer[1] == 'd' && buffer[2] == ' ')
        {
            char path_buffer[32];

            int i = 2;

            while (buffer[i] == ' ')
            {
                i ++;
            }

            int j = 0;

            while (buffer[i] != '\0')
            {
                path_buffer[j] = buffer[i];
                i++;
                j++;
            }

            path_buffer[j] = '\0';

            if (chdir(path_buffer) == -1)
            {
                eprintf("Unable to switch to `%s`\n", path_buffer);
            }

            continue;
        }

        bool do_time = false;

        if (buffer[0] == 't' && buffer[1] == 'i' && buffer[2] == 'm' && buffer[3] == 'e' && buffer[4] == ' ')
        {
            strcpy(buffer, &buffer[5]);
            do_time = true;
        }

        if (strcmp("quit", buffer) == 0)
        {
            break;
        }

        int buffer_index = 0;
        int argv_index = 0;
        argv[0] = buffer;

        while (buffer[buffer_index] != 0)
        {
            if (buffer[buffer_index] == ' ')
            {
                buffer[buffer_index] = 0;

                if (*argv[argv_index] != 0)
                {
                    argv_index++;
                }
                argv[argv_index] = &buffer[buffer_index + 1];
            }
            buffer_index++;
        }

        argv[++argv_index] = 0;

        if (*argv[0] == 0)
        {
            continue;
        }

        if (do_time)
        {
            run_exec_time(argv[0], argv, envp);
        }
        else
        {
            run_exec(argv[0], argv, envp);
        }
    }

    printf("Exiting Shell...\n");
}

void display_tag()
{
    char buffer[64];

    int pos = getcwd(buffer, 63);
    buffer[pos] = 0;

    printf("%s$> ", buffer);
}

int handle_redirect(char** argv)
{
    bool next_pipe_out = false;

    for (int i = 0; argv[i] != 0; i++)
    {
        if (!next_pipe_out)
        {
            if (strcmp(argv[i], ">") == 0)
            {
                // Found a cheveron
                argv[i] = 0;
                next_pipe_out = true;
            }
            else
            {
                next_pipe_out = false;
            }
        }
        else
        {
            int fd = open(argv[i], O_CREAT | O_TRUNC | O_WRONLY);

            if (fd < 0)
            {
                eprintf("Unable to open `%s`\n", argv[i]);

                return -1;
            }

            dup2(fd, 1);

            return fd;
        }
    }

    return 0;
}


int run_exec(char* exec, char** argv, char** envp)
{
    short pid = fork();

    if (pid == 0)
    {
        // handle_redirect(argv);
        execve(argv[0], (const char**)argv, (const char**)envp);

        if (argv[0][0] != '/')
        {
            char next_buffer[128];

            int i = 0;

            while (PATH[i] != 0)
            {
                next_buffer[i] = PATH[i];
                i++;
            }

            int j = 0;

            while (argv[0][j] != 0)
            {
                next_buffer[i] = argv[0][j];
                i++;
                j++;
            }

            next_buffer[i] = 0;

            execve(next_buffer, (const char**)argv, (const char**)envp);
        }

        eprintf("Unable to locate executable `%s`\n", argv[0]);

        exit(-1);
    }
    else
    {
        RUNNING_PID = pid;
        wait(0);
        RUNNING_PID = 0;

        return 0;
    }
}


int run_exec_time(char* exec, char** argv, char** envp)
{

    IS_TIMING = true;
    unsigned long start;
    unsigned long end;

    int fd = open("/dev/rtc0", O_RDONLY);

    if (fd < 0)
    {
        eprintf("Unable to open /dev/rtc0\n");
        return -1;
    }

    ioctl(fd, RTC_RD_TIMESTAMP, (unsigned long)&start);

    for (int i = 0; i < 10; IS_TIMING && i++)
    {
        if (run_exec(exec, argv, envp) < 0)
        {
            return -1;
        }
    }
    IS_TIMING = false;

    ioctl(fd, RTC_RD_TIMESTAMP, (unsigned long)&end);
    
    int avg = (end - start) / 10 / 1000000;

    printf("Average Runtime: %i ms\n", avg);

    return 0;
}