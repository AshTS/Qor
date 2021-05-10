#include "printf.h"

int main()
{
    int a = 0;
    int b = 1;
    int c = 1;

    for (int i = 0; i < 10; i++)
    {
        printf("fibb(%i) = %i\n", i, b);

        a = b;
        b = c;
        c = a + b;
    }

    return 4;
}