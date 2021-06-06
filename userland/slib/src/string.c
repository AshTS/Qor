#include "string.h"

char* strcat(char* s1, const char* s2)
{
    while (*(s1) != 0) {s1++;};
    while ((*(s1++) = *(s2++)));
    return s1;
}

char* strchr(const char* s, int c)
{
    char cmp = (char)(unsigned char)c;

    while (*s)
    {
        if (*s == cmp)
        {
            return s;
        }

        s++;
    }

    return 0;
}

int strcmp(const char* s1, const char* s2)
{
    while (*s1)
    {
        if (*s1 != *s2)
        {
            break;
        }

        s1++;
        s2++;
    }

    return *(const unsigned char*)s1 - *(const unsigned char*)s2;
}

char* strcpy(char* dest, const char* src)
{
    char* orig = dest;
    while ((*(dest++) = *(src++)));

    return orig;
}

int strlen(const char* s)
{
    int l = 0;

    while (*(s + l)) {l++;}

    return l;
}
