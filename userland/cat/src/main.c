#include "printf.h"
#include "stdbool.h"
#include "syscalls.h"

void cat_fd(int fd);

int main(int argc, char** argv)
{
    for (int i = 1; i < argc; i++)
    {
        int fd = open(argv[i], 0);
        
        if (fd < 0)
        {
            printf("cat: Cannot open file `%s`\n", argv[i]);
            return -1;
        }

        cat_fd(fd);

        close(fd);
    }   

    printf("\n");

    return 0;
}

void cat_fd(int fd)
{
    char buffer[1024];
    while (1)
    {
        int c = read(fd, buffer, 1024);

        if (!c)
        {
            break;
        }

        write(1, buffer, c);
    }
}