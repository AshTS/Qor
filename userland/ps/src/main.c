#include "printf.h"
#include "syscalls.h"
#include "string.h"

#define BUF_SIZE 1024

struct linux_dirent
{
   unsigned long  d_ino;
   unsigned long  d_off;
   unsigned short d_reclen;
   char           d_name[];
};


int main()
{
    char buffer[BUF_SIZE];
    
    // Open the /proc directory
    int proc_fd = open("/proc", O_RDONLY);

    if (proc_fd < 0)
    {
        printf("Unable to open /proc\n");
        return -1;
    }

    while (1)
    {
        int nread = getdents(proc_fd, buffer, BUF_SIZE);

        if (nread == -1)
        {
            printf("Error running getdents\n");
            return -1;
        }
        else if (nread == 0)
        {
            break;
        }
        else
        {
            for (int i = 0; i < nread; )
            {
                struct linux_dirent* d = (struct linux_dirent *) (buffer + i);
                i += d->d_reclen;

                if (((char*)d->d_name)[0] == '.')
                {
                    continue;
                }

                char path[256];

                if (d->d_ino != 0)
                {
                    sprintf(path, "/proc/%s/cmdline", (char *)d->d_name);
                    int fd = open(path, O_RDONLY);

                    if (fd < 0)
                    {
                        printf("Unable to open `%s`\n", path);
                        continue;
                    }

                    int length = strlen((char *)d->d_name);

                    printf("%s", (char *)d->d_name);

                    for (int i = 0; i < (5 - length); i++)
                    {
                        printf(" ");
                    }

                    int l = read(fd, path, 256);
                    
                    for (int i = 0; i < l; i++)
                    {
                        printf("%c", path[i] ? path[i] : ' ');
                    }

                    printf("\n");

                    close(fd);
                } 
            }

            break;
        }
    }   

    return 0;
}
