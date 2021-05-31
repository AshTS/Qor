#include "printf.h"
#include "syscalls.h"

int strcmp(char* A, char* B)
{
    while (*A)
    {
        if (*A != *B)
        {
            break;
        }

        A++;
        B++;
    }

    return *(const unsigned char*)A - *(const unsigned char*)B;
}

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
            return -1;
        }
        else
        {
            wait(0);
        }
    }

    printf("Exiting Shell...\n");
}