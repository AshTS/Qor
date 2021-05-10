#include "printf.h"

int bad_fibb(int i)
{
    if (i < 2)
        return 1;
    
    return bad_fibb(i - 1) + bad_fibb(i - 2);
}

int main()
{
    int a = 0;
    int b = 1;
    int c = 1;

    for (int i = 0; i < 100; i++)
    {
        printf("fibb(%i) = %i\n", i, b);
        printf("bad_fibb(%i) = %i\n", i, bad_fibb(i));

        a = b;
        b = c;
        c = a + b;
    }

    return 4;
}