#ifndef _STDDEF_H
#define _STDDEF_H

#include "stdint.h"

typedef intptr_t ptrdiff_t;
typedef uintmax_t size_t;

#define offsetof(structure, member) ((size_t)&(((structure *)0)->member))

#define NULL 0

#endif // _STDDEF_H