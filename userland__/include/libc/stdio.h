#ifndef _STDIO_H
#define _STDIO_H

#include "stdint.h"
#include "stddef.h"

typedef struct FILE
{
    uint32_t fd;
} FILE;

static FILE STDIN_FILE = {.fd = 0};
static FILE STDOUT_FILE = {.fd = 1};
static FILE STDERR_FILE = {.fd = 2};

#define stdin (&STDIN_FILE)
#define stdout (&STDOUT_FILE)
#define stderr (&STDERR_FILE)

int fprintf(FILE* stream, const char *format, ...);

int sprintf(char* s, const char *format, ...);

#define printf(...) fprintf (stdout, __VA_ARGS__)
#define eprintf(...) fprintf (stderr, __VA_ARGS__)

int fclose(FILE*);
FILE* fopen(const char*, const char*);
size_t fread(void*, size_t, size_t, FILE*);

#endif // _STDIO_H