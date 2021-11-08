#ifndef _PRINTF_H
#define _PRINTF_H

#include <libc/stdarg.h>
#include "syscalls.h"

extern int fprintf(int fd, const char* data, ...);
extern int sprintf(char* dest, const char* data, ...);

#define printf(...) fprintf (1, __VA_ARGS__)
#define eprintf(...) fprintf (2, __VA_ARGS__)

#endif // _PRINTF_H