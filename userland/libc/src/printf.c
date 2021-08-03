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

        *index = 0;
    }
}

// File Printf
int fprintf(int fd, const char* data, ...)
{
    va_list args;
    va_start(args, data);

    // Longer format specifiers
    char long_fmt_spec[16];
    unsigned int fmt_index = 0;

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

                fmt_index = 0;

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

                if (fmt_index > 1)
                {
                    int this_counter = 1;
                    while (long_fmt_spec[1]-- > '1')
                    {
                        this_counter *= 10;
                    }

                    if (this_counter > counter)
                    {
                        counter = this_counter; 
                    }
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
            long_fmt_spec[fmt_index++] = c;
            last_char = c;
            continue;
        }

        // Add to the buffer and possibly refresh the buffer
        printf_helper(buffer, &index, c, fd);
    }

    return index;
}

// Sput implementation for sprintf
void local_sput(const char* data, char** dest)
{
    int count = 0;
    while (*(data + count))
    {
        *((*(dest))++) = *(data + count);
        count++;
    }
}

// Helper function for sprintf
void sprintf_helper(char* buffer, unsigned int* index, char c, char** dest)
{
    buffer[((*index)++) % (PRINTF_BUFFER_LEN - 1)] = c;
    if (*index % (PRINTF_BUFFER_LEN - 1) == 0 || c == 0)
    {
        local_sput(buffer, dest);

        *index = 0;
    }
}

// String Printf
int sprintf(char* dest, const char* data, ...)
{
    va_list args;
    va_start(args, data);

    // Longer format specifiers
    char long_fmt_spec[16];
    unsigned int fmt_index = 0;

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

                fmt_index = 0;

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
                    sprintf_helper(buffer, &index, '-', &dest);
                }

                int counter = 1;
                while (counter <= i / 10)
                {
                    counter *= 10;
                }

                if (fmt_index > 1)
                {
                    int this_counter = 1;
                    while (long_fmt_spec[1]-- > '1')
                    {
                        this_counter *= 10;
                    }

                    if (this_counter > counter)
                    {
                        counter = this_counter; 
                    }
                }

                while (counter >= 1)
                {
                    sprintf_helper(buffer, &index, '0' + (i / counter) % 10, &dest);
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
                    sprintf_helper(buffer, &index, '-', &dest);
                }

                long counter = 1;
                while (counter <= i / 10)
                {
                    counter *= 10;
                }

                while (counter >= 1)
                {
                    sprintf_helper(buffer, &index, '0' + (i / counter) % 10, &dest);
                    counter /= 10;
                }

                next_format_specifier = false;
            }
            else if (c == 'p')
            {
                unsigned long i = va_arg(args, unsigned long);

                sprintf_helper(buffer, &index, '0', &dest);
                sprintf_helper(buffer, &index, 'x', &dest);

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
                        sprintf_helper(buffer, &index, '0' + digit, &dest);
                    }
                    else
                    {
                        sprintf_helper(buffer, &index, 'A' + (digit - 10), &dest);
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
                    sprintf_helper(buffer, &index, *(s++), &dest);
                }
                next_format_specifier = false;
            }
            else if (c == 'c')
            {
                int c = va_arg(args, int);

                sprintf_helper(buffer, &index, (char)c, &dest);
                next_format_specifier = false;
            }
            long_fmt_spec[fmt_index++] = c;
            last_char = c;
            continue;
        }

        // Add to the buffer and possibly refresh the buffer
        sprintf_helper(buffer, &index, c, &dest);
    }

    // Make sure the string gets null terminated
    *dest = 0;

    return index;
}