#include "printf.h"
#include "syscalls.h"
#include "string.h"

int main()
{
    while (1)
    {
        printf("$ ");

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
            execve(buffer);

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