#include "printf.h"
#include "syscalls.h"

#define BUF_SIZE 1024

struct linux_dirent
{
   unsigned long  d_ino;
   unsigned long  d_off;
   unsigned short d_reclen;
   char           d_name[];
};


int main(int argc, char** argv)
{
    char* dir;
    char buffer[BUF_SIZE];

    if (argc > 1)
    {
        dir = argv[1];
    }
    else
    {
        dir = ".";
    }

    int dir_fd = open(dir, O_RDONLY);

    if (dir_fd == -1)
    {
        eprintf("Cannot open directory `%s`\n", dir);
    }

    while (1)
    {
        int nread = getdents(dir_fd, buffer, BUF_SIZE);

        if (nread == -1)
        {
            eprintf("Error running getdents\n");
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

                if (d->d_ino != 0) printf("%s\n", (char *) d->d_name);
            }

            break;
        }
    }

    return 0;
}
