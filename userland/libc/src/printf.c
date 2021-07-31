#include "syscalls.h"
#include "printf.h"

#include <stdbool.h>

#define PRINTF_BUFFER_LEN 64

// Put implementation for printf
void local_put(const char* data, int fd)
{
    int count = 0;
    while (*(data + count))
    {
        count++;
    }

    write(fd, data, count);
}

// Helper function for printf
void printf_helper(char* buffer, unsigned int* index, char c, int fd)
{
    buffer[((*index)++) % (PRINTF_BUFFER_LEN - 1)] = c;
    if (*index % (PRINTF_BUFFER_LEN - 1) == 0 || c == 0)
    {
        local_put(buffer, fd);
    }
}

// File Printf
int fprintf(int fd, const char* data, ...)
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

    // Last character (for two byte format specifiers)
    char last_char = 0;

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
                last_char = c;
                continue;
            }

            last_char = c;
        }
        else
        {
            if (c == 'i')
            {
                int i = va_arg(args, int);

                if (i < 0)
                {
                    i = -i;
                    printf_helper(buffer, &index, '-', fd);
                }

                int counter = 1;
                while (counter <= i / 10)
                {
                    counter *= 10;
                }

                while (counter >= 1)
                {
                    printf_helper(buffer, &index, '0' + (i / counter) % 10, fd);
                    counter /= 10;
                }

                next_format_specifier = false;
            }
            else if (c == 'l')
            {
                next_format_specifier = true;
            }
            else if (c == 'd' && last_char == 'l')
            {
                long i = va_arg(args, long);

                if (i < 0)
                {
                    i = -i;
                    printf_helper(buffer, &index, '-', fd);
                }

                long counter = 1;
                while (counter <= i / 10)
                {
                    counter *= 10;
                }

                while (counter >= 1)
                {
                    printf_helper(buffer, &index, '0' + (i / counter) % 10, fd);
                    counter /= 10;
                }

                next_format_specifier = false;
            }
            else if (c == 'p')
            {
                unsigned long i = va_arg(args, unsigned long);

                printf_helper(buffer, &index, '0', fd);
                printf_helper(buffer, &index, 'x', fd);

                unsigned long counter = 1;
                while (counter <= i / 16)
                {
                    counter *= 16;
                }

                while (counter >= 1)
                {
                    char digit = (i / counter) % 16;

                    if (digit < 10)
                    {
                        printf_helper(buffer, &index, '0' + digit, fd);
                    }
                    else
                    {
                        printf_helper(buffer, &index, 'A' + (digit - 10), fd);
                    }
                    
                    counter /= 16;
                }
                next_format_specifier = false;
            }
            else if (c == 's')
            {
                const char* s = va_arg(args, const char*);

                while (*s)
                {
                    printf_helper(buffer, &index, *(s++), fd);
                }
                next_format_specifier = false;
            }
            else if (c == 'c')
            {
                int c = va_arg(args, int);

                printf_helper(buffer, &index, (char)c, fd);
                next_format_specifier = false;
            }
            last_char = c;
            continue;
        }
        

        // Add to the buffer and possibly refresh the buffer
        printf_helper(buffer, &index, c, fd);
    }

    return index;
}
