#include "syscalls.h"
#include "printf.h"

#include <stdbool.h>

#define PRINTF_BUFFER_LEN 5

// Put implementation for printf
void local_put(const char* data)
{
    put(data);
}

// Helper function for printf
void printf_helper(char* buffer, unsigned int* index, char c)
{
    buffer[((*index)++) % (PRINTF_BUFFER_LEN - 1)] = c;
    if (*index % (PRINTF_BUFFER_LEN - 1) == 0 || c == 0)
    {
        local_put(buffer);
    }
}

// Printf
int printf(const char* data, ...)
{
    va_list args;
    va_start(args, data);

    // Initialize the buffer
    char buffer[PRINTF_BUFFER_LEN];
    unsigned int index = 0;

    for (int i = 0; i < PRINTF_BUFFER_LEN; i++)
    {
        buffer[i] = 0;
    }

    // Flag for if the next character is to be a format specifier
    bool next_format_specifier = false;

    // Loop over every character in the input stream
    char c = 1;
    while (c)
    {
        // Get the next character
        c = *(data++);

        if (!next_format_specifier)
        {   
            if (c == '%')
            {
                next_format_specifier = true;
                continue;
            }
        }
        else
        {
            if (c == 'i')
            {
                int i = va_arg(args, int);

                if (i < 0)
                {
                    i = -i;
                    printf_helper(buffer, &index, '-');
                }

                int counter = 1;
                while (counter <= i / 10)
                {
                    counter *= 10;
                }

                while (counter >= 1)
                {
                    printf_helper(buffer, &index, '0' + (i / counter) % 10);
                    counter /= 10;
                }
            }
            else if (c == 's')
            {
                const char* s = va_arg(args, const char*);

                while (*s)
                {
                    printf_helper(buffer, &index, *(s++));
                }
            }
            else if (c == 'c')
            {
                int c = va_arg(args, int);

                printf_helper(buffer, &index, (char)c);
            }
            next_format_specifier = false;
            continue;
        }
        

        // Add to the buffer and possibly refresh the buffer
        printf_helper(buffer, &index, c);
    }

    return index;
}