#ifndef _SYSCALLS_H
#define _SYSCALLS_H

#define PROT_NONE 0
#define PROT_READ 1
#define PROT_WRITE 2
#define PROT_EXEC 4

#define MAP_SHARED 0
#define MAP_PRIVATE 0

#define MAP_ANONYMOUS 1
#define MAP_ANON MAP_ANONYMOUS

#define O_RDONLY 1
#define O_WRONLY 2
#define O_RDWR   3
#define O_APPEND 4
#define O_TRUNC  8
#define O_CREAT  16
#define O_EXCL   32

#define SEEK_SET 1
#define SEEK_CUR 2
#define SEEK_END 4

#define RTC_RD_TIME      0x7009
#define RTC_RD_TIMESTAMP 0x70FF

#define FB_FLUSH         0x46FF

struct time_repr
{
    long tv_sec;
    long tv_nsec;
};

extern unsigned int exit(int val);
extern unsigned int write(int fd, void* buffer, int size);
extern unsigned int open(const char* name, int open_mode);
extern unsigned int close(int fd);
extern unsigned int read(int fd, void *buffer, int size);
extern unsigned int fork();
extern unsigned int execve(const char *path, const char **argv, const char **envp);
extern unsigned int wait(int *wstatus);
extern unsigned int getcwd(void *buffer, int size);
extern void *mmap(void *start, int length, int prot, int flags, int fd, int off);
extern unsigned int munmap(void *buffer, int size);
extern unsigned int getdents(int fd, void *dirents, unsigned int count);
extern unsigned int chdir(const char *path);
extern unsigned int mkdir(const char *path, unsigned short mode);
extern unsigned int lseek(int fd, unsigned int offset, int whence);
extern unsigned int pipe(int fds[2]);
extern unsigned int dup2(int old_fd, int new_fd);
extern unsigned int dup(int old);
extern unsigned int ioctl(unsigned int fd, unsigned int cmd, unsigned long arg);
extern unsigned int nanosleep(struct time_repr* req, struct time_repr* rem);
extern int pause();
extern int kill(short pid, int signal);
extern int sigaction(int signal, void* new_action, void* old_action);
extern void sigreturn();
extern void sync();

#endif // _SYSCALLS_H