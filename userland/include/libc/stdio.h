#ifndef _STDIO_H
#define _STDIO_H

#define stdin 0
#define stdout 1
#define stderr 2

typedef unsigned int FILE_PTR;

int fprintf(FILE_PTR stream, const char *format, ...);

int sprintf(char* s, const char *format, ...);

#define printf(...) fprintf (stdout, __VA_ARGS__)
#define eprintf(...) fprintf (stderr, __VA_ARGS__)

#endif // _STDIO_H