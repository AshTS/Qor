#include "printf.h"
#include "string.h"

int main()
{
    char dest[64];
    char* src = "Hello World!";

    dest[0] = ':';
    dest[1] = 0;

    strcat(dest, src);

    printf("`%s`\n", dest);


    return 0;
}
