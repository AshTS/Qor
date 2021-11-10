#include "printf.h"
#include "syscalls.h"

int main()
{
    char buffer[64];

    int pos = getcwd(buffer, 63);
    buffer[pos] = 0;

    printf("%s\n", buffer);

    return 0;
}
