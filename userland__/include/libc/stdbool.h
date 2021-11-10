#ifndef _STDBOOL_H
#define _STDBOOL_H

#include "stdint.h"

#ifdef _Bool
#define bool _Bool
#else
#define bool uint8_t
#endif

#define true 1
#define false 0

#define __bool_true_false_are_defined 1

#endif