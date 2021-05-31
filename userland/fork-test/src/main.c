#include "printf.h"
#include "syscalls.h"

int main()
{
    printf("Trying to fork!\n");
    int result = fork();
    if (result == 0)
    {
        execve("/bin/prog");
    }

    return 0;
}
