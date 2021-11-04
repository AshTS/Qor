#ifndef _STDARG_H
#define _STDARG_H

#include "stddef.h"

typedef __builtin_va_list va_list;

#define internal_va_size(v) (((sizeof(v) + sizeof(int) - 1) / (sizeof(int))) * (sizeof(int)))

#define va_start(ap, var) __builtin_va_start(ap, var)

#define va_arg(ap, type) __builtin_va_arg(ap, type)

#define va_end(ap) __builtin_va_end(ap)


#endif // _STDARG_H