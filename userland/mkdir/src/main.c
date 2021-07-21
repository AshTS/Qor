#include "printf.h"
#include "syscalls.h"

int main(int argc, char** argv)
{
    if (argc < 2)
    {
        printf("mkdir requires atleast one argument\n");
    }
    else
    {
        if (mkdir(argv[1], 0x1FF) == -1)
        {
            printf("Unable to create directory `%s`\n", argv[1]);
            return -1;
        }
    }
    return 0;
}
