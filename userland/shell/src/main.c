#include "printf.h"
#include "syscalls.h"
#include "string.h"

void display_tag();

int main()
{
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

            execve(argv[0], argv, envp);

            printf("Unable to open file `%s`\n", buffer);

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