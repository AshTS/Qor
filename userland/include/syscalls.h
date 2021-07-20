#ifndef _SYSCALLS_H
#define _SYSCALLS_H

#define PROT_NONE 0
#define PROT_READ 1
#define PROT_WRITE 2
#define PROT_EXEC 4

#define MAP_SHARED 0
#define MAP_PRIVATE 0

#define MAP_ANONYMOUS 1

extern unsigned int exit(int val);
extern unsigned int write(int fd, void* buffer, int size);
extern unsigned int open(const char* name, int mode);
extern unsigned int close(int fd);
extern unsigned int read(int fd, void* buffer, int size);
extern unsigned int fork();
extern unsigned int execve(const char* path, const char** argv, const char** envp);
extern unsigned int wait(int* wstatus);
extern unsigned int getcwd(void* buffer, int size);
extern void* mmap(void* start, int length, int prot, int flags, int fd, int off);
extern unsigned int munmap(void* buffer, int size);
extern unsigned int getdents(int fd, void* dirents, unsigned int count);
extern unsigned int chdir(const char* path);

#endif // _SYSCALLS_H