#include "printf.h"
#include "syscalls.h"
#include "string.h"

int main(int argc, char** argv)
{
    int fd = open("/dev/disp", O_WRONLY | O_TRUNC);

    if (fd == -1)
    {
        printf("Unable to open `open.txt`\n");
        return -1;
    }

    char* data = "Hello from a userspace program!\0";
    int length = strlen(data);

    write(fd, data, length);

    close(fd);
}