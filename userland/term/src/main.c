#include "syscalls.h"
#include "printf.h"

int main()
{
    open("/dev/null", O_RDONLY);
    int out = open("/dev/tty0", O_WRONLY);
    int err = open("/dev/tty0", O_WRONLY);

    int fd = open("/dev/tty0", O_RDONLY);

    close(0);

    int fds[2];
    
    pipe(fds);

    if (fork() == 0)
    {
        const char* name = "/bin/shell";
        const char* argv[2];

        argv[0] = name;
        argv[1] = 0;

        const char* envp[1];

        envp[0] = 0;

        execve("/bin/shell", argv, envp);
    }

    char buffer[256];
    int buffer_index = 0;

    while (1)
    {
        char c; 
        if (read(fd, &c, 1))
        {
            write(out, &c, 1);

            if (c == 10 || c == 13)
            {
                write(fds[1], buffer, buffer_index + 1);
                buffer_index = 0;
            }
            else if (c == 8 || c == 127)
            {
                if (buffer_index > 0)
                {
                    buffer_index--;
                }
            }
            else
            {
                buffer[buffer_index] = c;
                buffer_index++;

                if (buffer_index >= 256)
                {
                    buffer_index = 255;
                }
            }
        }
    }

    return 0;
}
