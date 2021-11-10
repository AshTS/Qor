#ifndef _ASSERT_H
#define _ASSERT_H

#ifndef NDEBUG
#include "stdio.h"
#include "sys/syscalls.h"
#define assert(val) (((int)(val)) ? ((void)0) : (eprintf("Assertation Failed: "__FILE__":%i: %s: Assertion `"#val"` failed.\n", __LINE__, __func__), exit(1))) 
#else
#define assert(ignore) ((void)0)
#endif

#endif // _ASSERT_H