#ifndef _STRING_H
#define _STRING_H

#include "stddef.h"

#define NULL 0

// Copies bytes from memory area s2 into s1, stopping after the first occurrence
// of byte c (converted to an unsigned char) is copied, or after n bytes are
// copied, whichever comes first. If copying takes place between objects that
// overlap, the behaviour is undefined.
//
// Returns a pointer to the byte after the copy of c in s1, or a null pointer if
// c was not found in the first n bytes of s2.
void* memccpy(void*, const void*, int, size_t);

// Locates the first occurrence of c (converted to an unsigned char) in the
// initial n bytes (each interpreted as unsigned char) of the object pointed to
// by s.
// 
// Returns a pointer to the located byte, or a null pointer if the byte does not
// occur in the object.
void* memchr(const void*, int, size_t);

// Compares the first n bytes (each interpreted as unsigned char) of the object 
// pointed to by s1 to the first n bytes of the object pointed to by s2.
// 
// The sign of a non-zero return value is determined by the sign of the
// difference between the values of the first pair of bytes (both interpreted as
// type unsigned char) that differ in the objects being compared.
// 
// Returns an integer greater than, equal to or less than 0, if the object
// pointed to by s1 is greater than, equal to or less than the object pointed to
// by s2 respectively.
int memcmp(const void*, const void*, size_t);

// Copies n bytes from the object pointed to by s2 into the object pointed to by
// s1. If copying takes place between objects that overlap, the behaviour is
// undefined.
//
// Returns s1; no return value is reserved to indicate an error.
void* memcpy(void*, const void*, size_t);

// Copies n bytes from the object pointed to by s2 into the object pointed to by
// s1. Copying takes place as if the n bytes from the object pointed to by s2
// are first copied into a temporary array of n bytes that does not overlap the
// objects pointed to by s1 and s2, and then the n bytes from the temporary
// array are copied into the object pointed to by s1.
//
// Returns s1; no return value is reserved to indicate an error.
void *memmove(void*, const void*, size_t);

// Copies c (converted to an unsigned char) into each of the first n bytes of
// the object pointed to by s.
//
// Returns s; no return value is reserved to indicate an error.
void* memset(void*, int, size_t);

// Appends a copy of the string pointed to by s2 (including the terminating null
// byte) to the end of the string pointed to by s1. The initial byte of s2
// overwrites the null byte at the end of s1. If copying takes place between
// objects that overlap, the behaviour is undefined.
//
// Returns s1; no return value is reserved to indicate an error.
char* strcat(char*, const char*);

// Locates the first occurrence of c (converted to an unsigned char) in the
// string pointed to by s. The terminating null byte is considered to be part
// of the string.
//
// Returns a pointer to the byte, or a null pointer if the byte was not found.
char* strchr(const char*, int);

// Compares the string pointed to by s1 to the string pointed to by s2.
//
// The sign of a non-zero return value is determined by the sign of the
// difference between the values of the first pair of bytes (both interpreted
// as type unsigned char) that differ in the strings being compared.
//
// Returns an integer greater than, equal to or less than 0, if the string
// pointed to by s1 is greater than, equal to or less than the string pointed to
// by s2 respectively.
int strcmp(const char*, const char*);

// Copies the string pointed to by s2 (including the terminating null byte) into
// the array pointed to by s1. If copying takes place between objects that
// overlap, the behaviour is undefined.
// 
// Returns s1; no return value is reserved to indicate an error.
char* strcpy(char*, const char*);

// Computes the length of the maximum initial segment of the string pointed to
// by s1 which consists entirely of bytes not from the string pointed to by s2.
// 
// Returns the length of the computed segment of the string pointed to by s1;
// no return value is reserved to indicate an error.
size_t strcspn(const char*, const char*);

// Returns a pointer to a new string, which is a duplicate of the string pointed
// to by s1. The returned pointer can be passed to free(). A null pointer is
// returned if the new string cannot be created.
//
// Returns a pointer to a new string on success. Otherwise it returns a null
// pointer and sets errno to indicate the error.
char* strdup(const char*);

// Maps the error number in errnum to a locale-dependent error message string
// and returns a pointer thereto. The string pointed to must not be modified by
// the program, but may be overwritten by a subsequent call to strerror() or
// perror().
//
// The contents of the error message strings returned by strerror() should be
// determined by the setting of the LC_MESSAGES category in the current locale.
// 
// The implementation will behave as if no function defined in this
// specification calls strerror().
// 
// The strerror() function will not change the setting of errno if successful.
// 
// Because no return value is reserved to indicate an error, an application
// wishing to check for error situations should set errno to 0, then call
// strerror(), then check errno.
// 
// This interface need not be reentrant.
// 
// Upon successful completion, strerror() returns a pointer to the generated
// message string. On error errno may be set, but no return value is reserved to
// indicate an error.
char* strerror(int);

// Computes the number of bytes in the string to which s points, not including
// the terminating null byte.
//
// Returns the length of s; no return value is reserved to indicate an error.
size_t strlen(const char*);

// Appends not more than n bytes (a null byte and bytes that follow it are not
// appended) from the array pointed to by s2 to the end of the string pointed to
// by s1. The initial byte of s2 overwrites the null byte at the end of s1. A
// terminating null byte is always appended to the result. If copying takes
// place between objects that overlap, the behaviour is undefined.
//
// Returns s1; no return value is reserved to indicate an error.
char* strncat(char*, const char*, size_t);

// Compares not more than n bytes (bytes that follow a null byte are not
// compared) from the array pointed to by s1 to the array pointed to by s2.
//
// The sign of a non-zero return value is determined by the sign of the
// difference between the values of the first pair of bytes (both interpreted as
// type unsigned char) that differ in the strings being compared.
//
// Returns an integer greater than, equal to or less than 0, if the possibly
// null-terminated array pointed to by s1 is greater than, equal to or less than
// the possibly null-terminated array pointed to by s2 respectively.
int strncmp(const char*, const char*, size_t);

// Copies not more than n bytes (bytes that follow a null byte are not copied)
// from the array pointed to by s2 to the array pointed to by s1. If copying
// takes place between objects that overlap, the behaviour is undefined.
//
// If the array pointed to by s2 is a string that is shorter than n bytes, null
// bytes are appended to the copy in the array pointed to by s1, until n bytes
// in all are written.
// 
// Returns s1; no return value is reserved to indicate an error.
char* strncpy(char*, const char*, size_t);

// Locates the first occurrence in the string pointed to by s1 of any byte from
// the string pointed to by s2.
//
// Returns a pointer to the byte or a null pointer if no byte from s2 occurs in
// s1.
char* strpbrk(const char*, const char*);

// Locates the last occurrence of c (converted to a char) in the string pointed
// to by s. The terminating null byte is considered to be part of the string.
//
// Returns a pointer to the byte or a null pointer if c does not occur in the string.
char* strrchr(const char*, int);

// Computes the length of the maximum initial segment of the string pointed to
// by s1 which consists entirely of bytes from the string pointed to by s2.
//
// Returns the length of s1; no return value is reserved to indicate an error.
size_t strspn(const char*, const char*);

// Locates the first occurrence in the string pointed to by s1 of the sequence
// of bytes (excluding the terminating null byte) in the string pointed to by
// s2.
// 
// Returns a pointer to the located string or a null pointer if the string is
// not found.
// If s2 points to a string with zero length, the function returns s1.
char* strstr(const char*, const char*);

// A sequence of calls to strtok() breaks the string pointed to by s1 into a
// sequence of tokens, each of which is delimited by a byte from the string
// pointed to by s2. The first call in the sequence has s1 as its first
// argument, and is followed by calls with a null pointer as their first
// argument. The separator string pointed to by s2 may be different from call
// to call.
// The first call in the sequence searches the string pointed to by s1 for the 
// first byte that is not contained in the current separator string pointed to
// by s2. If no such byte is found, then there are no tokens in the string
// pointed to by s1 and strtok() returns a null pointer. If such a byte is
// found, it is the start of the first token.
// 
// The strtok() function then searches from there for a byte that is contained
// in the current separator string. If no such byte is found, the current token
// extends to the end of the string pointed to by s1, and subsequent searches
// for a token will return a null pointer. If such a byte is found, it is
// overwritten by a null byte, which terminates the current token. The strtok()
// function saves a pointer to the following byte, from which the next search
// for a token will start.
// 
// Each subsequent call, with a null pointer as the value of the first argument,
// starts searching from the saved pointer and behaves as described above.
// 
// The implementation will behave as if no function defined in this document
// calls strtok().
// 
// The strtok() interface need not be reentrant.
// 
// The function strtok_r() considers the null-terminated string s as a sequence
// of zero or more text tokens separated by spans of one or more characters from
// the separator string sep. The argument lasts points to a user-provided
// pointer which points to stored information necessary for strtok_r() to
// continue scanning the same string.
// 
// In the first call to strtok_r(), s points to a null-terminated string, sep to
// a null-terminated string of separator characters and the value pointed to by
// lasts is ignored. The function strtok_r() returns a pointer to the first
// character of the first token, writes a null character into s immediately
// following the returned token, and updates the pointer to which lasts points.
// 
// In subsequent calls, s is a NULL pointer and lasts will be unchanged from the
// previous call so that subsequent calls will move through the string s,
// returning successive tokens until no tokens remain. The separator string sep
// may be different from call to call. When no token remains in s, a NULL
// pointer is returned.
// 
// Upon successful completion, strtok() returns a pointer to the first byte of a
// token. Otherwise, if there is no token, strtok() returns a null pointer.
// The function strtok_r() returns a pointer to the token found, or a NULL
// pointer when no token is found.
char* strtok(char*, const char*);

// A sequence of calls to strtok() breaks the string pointed to by s1 into a
// sequence of tokens, each of which is delimited by a byte from the string
// pointed to by s2. The first call in the sequence has s1 as its first
// argument, and is followed by calls with a null pointer as their first
// argument. The separator string pointed to by s2 may be different from call
// to call.
// The first call in the sequence searches the string pointed to by s1 for the 
// first byte that is not contained in the current separator string pointed to
// by s2. If no such byte is found, then there are no tokens in the string
// pointed to by s1 and strtok() returns a null pointer. If such a byte is
// found, it is the start of the first token.
// 
// The strtok() function then searches from there for a byte that is contained
// in the current separator string. If no such byte is found, the current token
// extends to the end of the string pointed to by s1, and subsequent searches
// for a token will return a null pointer. If such a byte is found, it is
// overwritten by a null byte, which terminates the current token. The strtok()
// function saves a pointer to the following byte, from which the next search
// for a token will start.
// 
// Each subsequent call, with a null pointer as the value of the first argument,
// starts searching from the saved pointer and behaves as described above.
// 
// The implementation will behave as if no function defined in this document
// calls strtok().
// 
// The strtok() interface need not be reentrant.
// 
// The function strtok_r() considers the null-terminated string s as a sequence
// of zero or more text tokens separated by spans of one or more characters from
// the separator string sep. The argument lasts points to a user-provided
// pointer which points to stored information necessary for strtok_r() to
// continue scanning the same string.
// 
// In the first call to strtok_r(), s points to a null-terminated string, sep to
// a null-terminated string of separator characters and the value pointed to by
// lasts is ignored. The function strtok_r() returns a pointer to the first
// character of the first token, writes a null character into s immediately
// following the returned token, and updates the pointer to which lasts points.
// 
// In subsequent calls, s is a NULL pointer and lasts will be unchanged from the
// previous call so that subsequent calls will move through the string s,
// returning successive tokens until no tokens remain. The separator string sep
// may be different from call to call. When no token remains in s, a NULL
// pointer is returned.
// 
// Upon successful completion, strtok() returns a pointer to the first byte of a
// token. Otherwise, if there is no token, strtok() returns a null pointer.
// The function strtok_r() returns a pointer to the token found, or a NULL
// pointer when no token is found.
char* strtok_r(char*, const char*, char**);

#endif // _STRING_H