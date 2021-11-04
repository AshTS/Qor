#ifndef _STDINT_H
#define _STDINT_H

typedef char int8_t;
typedef char int_fast8_t;
typedef char int_least8_t;

typedef unsigned char uint8_t;
typedef unsigned char uint_fast8_t;
typedef unsigned char uint_least8_t;


typedef short int16_t;
typedef short int_fast16_t;
typedef short int_least16_t;

typedef unsigned short uint16_t;
typedef unsigned short uint_fast16_t;
typedef unsigned short uint_least16_t;


typedef int int32_t;
typedef int int_fast32_t;
typedef int int_least32_t;

typedef unsigned int uint32_t;
typedef unsigned int uint_fast32_t;
typedef unsigned int uint_least32_t;


typedef long int64_t;
typedef long int_fast64_t;
typedef long int_least64_t;

typedef unsigned long uint64_t;
typedef unsigned long uint_fast64_t;
typedef unsigned long uint_least64_t;


typedef long intmax_t;
typedef long intptr_t;

typedef unsigned long uintmax_t;
typedef unsigned long uintptr_t;

#define __INT64_C(num) c ## L
#define __UINT64_C(num) c ## UL

#define INT8_C(c) c
#define INT16_C(c) c
#define INT32_C(c) c
#define INT64_C(c) c ## L

#define INTMAX_C(c) c ## L

#define UINT8_C(c) c
#define UINT16_C(c) c
#define UINT32_C(c) c
#define UINT64_C(c) c ## UL

#define INTMAX_C(c) c ## L

#define INT8_MIN (-128)
#define INT16_MIN (-32767-1)
#define INT32_MIN (-2147483647-1)
#define INT64_MIN (-__INT64_C(9223372036854775807)-1)
#define INTPTR_MIN (-__INT64_C(9223372036854775807)-1)
#define INTMAX_MIN (-__INT64_C(9223372036854775807)-1)
  
#define INT8_MAX (127)
#define INT16_MAX (32767)
#define INT32_MAX (2147483647)
#define INT64_MAX (__INT64_C(9223372036854775807))
#define INTPTR_MAX (__INT64_C(9223372036854775807))
#define INTMAX_MAX (__INT64_C(9223372036854775807))

#define UINT8_MAX (255)
#define UINT16_MAX (65535)
#define UINT32_MAX (4294967295U)
#define UINT64_MAX (__UINT64_C(18446744073709551615))
#define UINTPTR_MAX (__UINT64_C(18446744073709551615))
#define UINTMAX_MAX (__UINT64_C(18446744073709551615))

#define INT_LEAST8_MIN (-128)
#define INT_LEAST16_MIN (-32767-1)
#define INT_LEAST32_MIN (-2147483647-1)
#define INT_LEAST64_MIN (-__INT64_C(9223372036854775807)-1)
#define INT_LEAST8_MAX (127)
#define INT_LEAST16_MAX (32767)
#define INT_LEAST32_MAX (2147483647)
#define INT_LEAST64_MAX (__INT64_C(9223372036854775807))

#define UINT_LEAST8_MAX (255)
#define UINT_LEAST16_MAX (65535)
#define UINT_LEAST32_MAX (4294967295U)
#define UINT_LEAST64_MAX (__UINT64_C(18446744073709551615))

#endif // _STDINT_H