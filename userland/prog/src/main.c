#include "printf.h"
#include "syscalls.h"
#include "string.h"
#include "malloc.h"

#define RTC_RD_TIME 0x7009

void* memcpy( void* dest, const void* src, int count )
{
    for (int i = 0; i < count; i++)
    {
        *((char*)dest + i) = *((char*)src + i);
    }

    return dest;
}

int main(int argc, char** argv)
{
    printf("Sending Signal\n");

    kill(4, 15);
    /*
    int fd = open("/dev/rtc0", O_RDONLY);

    if (fd < 0)
    {
        printf("Unable to open /dev/rtc0\n");
        return -1;
    }

    struct rtc_time data;

    for (int i = 0; i < 3000; i++)
    {
        int disp = open("/dev/disp", O_WRONLY);

        if (disp < 0)
        {
            printf("Unable to open /dev/disp\n");
            return -1;
        }

        ioctl(fd, RTC_RD_TIME, &data);

        char* months[] = {"Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"};

        fprintf(disp, "\x1B[0;0H%02i:%02i:%02i %i %s %i\n", data.tm_hour, data.tm_min, data.tm_sec, data.tm_mday, months[data.tm_mon % 12], data.tm_year + 1900);

        struct time_repr req = { .tv_sec=0, .tv_nsec=500000000 };
        struct time_repr rem;

        nanosleep(&req, &rem);
        close(disp);
    }

    close(fd);*/

    printf("Signal Sent\n");

    return 0;
}
