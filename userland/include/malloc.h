#ifndef _MALLOC_H
#define _MALLOC_H

extern void* malloc(unsigned int size);
extern void free(void* ptr);

extern void dump();

#endif // _MALLOC_H