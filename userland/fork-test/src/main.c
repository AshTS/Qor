#include "printf.h"
#include "syscalls.h"

int main()
{
    printf("Trying to fork!\n");
    int result = fork();

    printf("Got Result: %i\n", result);

    return 0;
}
