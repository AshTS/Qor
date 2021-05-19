#include "printf.h"
#include "syscalls.h"

int main()
{
    int fd = open("/root/test.txt", 0);
    printf("Got fd: %i\n", fd);

    if (fd != -1)
    {
        close(fd);
    }

    return 0;
}
