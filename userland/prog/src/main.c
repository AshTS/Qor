#include "printf.h"
#include "syscalls.h"

int main()
{
    printf("Stdin Test Program\n");

    printf("Enter Some Text: ");

    char buffer[64];
    buffer[63] = 0;

    while (read(0, buffer, 63) == 0) {}

    printf("Got Text: `%s`\n", buffer);


}