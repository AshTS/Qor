#ifndef _SYSCALLS_H
#define _SYSCALLS_H

extern unsigned int write(int fd, void* buffer, int size);
extern unsigned int open(const char* name, int mode);
extern unsigned int close(int fd);
extern unsigned int read(int fd, void* buffer, int size);
extern unsigned int fork();
extern unsigned int execve(const char* path);
extern unsigned int wait(int* wstatus);
extern unsigned int getcwd(void* buffer, int size);

#endif // _SYSCALLS_H