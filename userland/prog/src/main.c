#include "printf.h"

int factorial(int v);

int main()
{
    for (int i = 0; i < 10; i++)
    {
        printf("%i!\t = %i\n", i, factorial(i));
    }
    return 0;
}

int factorial(int v)
{
    if (v < 2)
    {
        return 1;
    }

    return factorial(v - 1) * v;
}