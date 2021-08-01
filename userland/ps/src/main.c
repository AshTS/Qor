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

int string_to_int(char* s);
void display_pid(int pid);

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

                int pid = string_to_int((char*)d->d_name);

                char path[256];

                if (d->d_ino != 0)
                {
                    display_pid(pid);
                } 
            }

            break;
        }
    }   

    return 0;
}


int string_to_int(char* s)
{
    int result = 0;
    while (*s >= '0' && *s <= '9')
    {
        result *= 10;
        result += *s - '0';
        s++;
    }

    return result;
}

int get_mem_usage(int pid)
{
    char path[256];

    sprintf(path, "/proc/%i/statm", pid);
    int fd = open(path, O_RDONLY);

    if (fd < 0)
    {
        // printf("\nUnable to open `%s`\n", path);
        return -1;
    }

    int l = read(fd, path, 255);
    path[l] = 0;

    close(fd);

    return string_to_int(path);
}

int render_size(int size)
{
    char endings[] = {'K', 'M', 'G', 'T'};
    char i = 0;

    while (size > 2048)
    {
        size /= 1024;
        i++;
    }

    return printf("%i%c", size, endings[i]);
}

void display_pid(int pid)
{
    char path[256];

    sprintf(path, "/proc/%i/cmdline", pid);
    int fd = open(path, O_RDONLY);

    if (fd < 0)
    {
        printf("Unable to open `%s`\n", path);
        return;
    }

    int length = printf("%i", pid);

    for (int i = 0; i < (3 - length); i++)
    {
        length += printf(" ");
    }

    int l = read(fd, path, 256);
    for (int i = 0; i < l; i++)
    {
        if (path[i] == 0) { break; }
        printf("%c", path[i]);
        length += 1;
    }

    for (int i = 0; i < (12 - length); i++)
    {
        length += printf(" ");
    }

    int mem_usage = get_mem_usage(pid) * 4;

    length += render_size(mem_usage);



    printf("\n");

    close(fd);
}