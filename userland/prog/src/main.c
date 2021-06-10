#include "printf.h"
#include "syscalls.h"

int main(int argc, char** argv)
{
    printf("Argc: %i\n", argc);

    for (int i = 0; i < argc; i++)
    {
        printf("`%s`\n", argv[i]);
    }
}