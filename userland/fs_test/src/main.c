#include "printf.h"
#include "syscalls.h"

int main()
{
    int fd = open("/root/test.txt", 0);
    printf("Got fd: %i\n", fd);

    char buffer[1024];

    if (fd != -1)
    {
        read(fd, buffer, 1024);

        printf("Got Text: `%s`", buffer);

        close(fd);
    }

    execve("/bin/prog");

    return 0;
}
