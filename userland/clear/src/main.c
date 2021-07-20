#include "printf.h"

int main()
{
    printf("\x1B[0;0f");

    for (int i = 0; i < 512; i++)
    {
        printf("                ");
    }

    printf("\x1B[0;0f");

    return 0;
}
