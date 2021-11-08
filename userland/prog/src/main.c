#include "libc/string.h"
#include "libc/stdio.h"

#include "libc/sys/syscalls.h"

#include "libgraphics.h"
#include "time.h"

#define MAX(a, b) ((a) < (b) ? (b) : (a))
#define MIN(a, b) ((a) > (b) ? (b) : (a))

#define PI 3.1415926535897932384626433832795028841971693993751058209749445923078164062862089986280348253421170679
#define PI2 PI / 2.0
#define PI4 PI / 4.0

#define ABS(a) ((a) < 0 ? (-(a)) : (a))

#define ITER(i) i

int rand_mem = PI * 0xFFFFFFFF;
float scale = 200.0;

struct Pixel grid_shader(int x, int y);
int rand();

int main(int argc, char** argv)
{
    init_framebuffer();

    for (int c = 0; c < 1000; c++)
    {
        // run_individual_shader(grid_shader);

        struct Pixel* buffer = get_framebuffer();

        if (buffer == 0)
        {
            return -1;
        }

        int x = rand() % 640;
        int y = rand() % 480;

        if (x < 0) { x = -x; }
        if (y < 0) { y = -y; }

        printf("x: %i, y: %i\n", x, y);

        buffer[compute_location(x, y)] = COLOR_WHITE;

        flush_framebuffer();
    }

    close_framebuffer();
}

int rand()
{
    int fd = open("/dev/rtc0", O_RDONLY);

    if (fd < 0)
    {
        printf("Unable to open /dev/rtc0\n");
        return -1;
    }

    unsigned long data;

    ioctl(fd, RTC_RD_TIMESTAMP, (unsigned long)&data);

    close(fd);

    rand_mem ^= data >> 2;

    return rand_mem;
}

struct Pixel grid_shader(int x, int y)
{
    x -= 320;
    y -= 240;

    if (x % (int)scale == 0 || y % (int)scale == 0)
    {
        return COLOR_WHITE;
    }
    else if (x % ((int)scale / 4) == 0 || y % ((int)scale / 4) == 0)
    {
        return COLOR_GREY;
    }

    return COLOR_BLACK;
}
