#include "printf.h"
#include "syscalls.h"
#include "string.h"
#include "signals.h"

#include <stdbool.h>

char* PATH = "/bin/";

void display_tag();

int handle_redirect(char** argv);

int main()
{
    // Setup the handler for SIGINT
    struct sigaction new;
    struct sigaction old;

    new.sa_flags = 0;
    new.sa_handler = SIG_IGN;

    sigaction(SIGINT, &new, &old);

    char* envp[1];
    envp[0] = 0;

    char* argv[64];
    argv[0] = 0;

    while (1)
    {
        display_tag();

        char buffer[64];
        while (1)
        {
            int count = read(0, buffer, 63);

            if (count == 0) continue;

            buffer[count - 1] = 0;

            break;
        }

        if (buffer[0] == 'c' && buffer[1] == 'd')
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
                printf("Unable to switch to `%s`\n", path_buffer);
            }

            continue;
        }

        if (strcmp("quit", buffer) == 0)
        {
            break;
        }

        if (fork() == 0)
        {
            int buffer_index = 0;
            int argv_index = 0;
            argv[0] = buffer;

            while (buffer[buffer_index] != 0)
            {
                if (buffer[buffer_index] == ' ')
                {
                    buffer[buffer_index] = 0;
                    argv_index++;
                    argv[argv_index] = &buffer[buffer_index + 1];
                }
                buffer_index++;
            }

            handle_redirect(argv);

            execve(argv[0], argv, envp);

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

                execve(next_buffer, argv, envp);
            }

            printf("Unable to locate executable `%s`\n", buffer);

            return -1;
        }
        else
        {
            wait(0);
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
                printf("Unable to open `%s`\n", argv[i]);

                return -1;
            }

            dup2(fd, 1);

            return fd;
        }
    }
}