#include "printf.h"

#define PREC 10000
#define ZOOM 1
#define CENTER_X -7500	// -0.75
#define CENTER_Y -0   // 0.0

struct Pixel
{
    char r;
    char g;
    char b;
    char a;
};

struct Complex
{
    long real;
    long imag;
};

char check_pixel(int x, int y, int zoom);

struct Complex iterate(struct Complex z, struct Complex c);

long mag2(struct Complex z);

int main()
{
    printf("Fractal Plotter!\n");

    int fb_fd = open("/dev/fb0", O_WRONLY);

    if (fb_fd == -1)
    {
        eprintf("Unable to open /dev/fb0\n");
        return -1;
    }
    struct Pixel* framebuffer = mmap(0, 640 * 480 * 4, PROT_READ | PROT_WRITE, 0, fb_fd, 0);

    struct Pixel color;

    color.r = 255;
    color.g = 255;
    color.b = 255;
    color.a = 255;

    for (int x = 0; x < 640; x++)
    {
        for (int y = 0; y < 480; y++)
        {
            char c = check_pixel(x, y, ZOOM);

            color.r = c;
            color.g = c;
            color.b = c;

            framebuffer[x + 640 * y] = color;
        }
    }

    munmap(framebuffer, 640 * 480 * 4);

    close(fb_fd);

    return 0;
}

char check_pixel(int x, int y, int zoom)
{
    struct Complex z;

    z.real = 0;
    z.imag = 0;

    struct Complex c;

    c.real = (x - 320) * PREC / 200 / zoom + CENTER_X;
    c.imag = (y - 240) * PREC / 200 / zoom + CENTER_Y;

    char val = 255;

    while (val != 0)
    {
        z = iterate(z, c);

        if (mag2(z) > 4 * PREC)
        {
            break;
        }

        val -= 1;
    }

    return val;
}

struct Complex iterate(struct Complex z, struct Complex c)
{
    struct Complex sqr;

    sqr.real = (z.real * z.real - z.imag * z.imag) / PREC;
    sqr.imag = (z.real * z.imag * 2) / PREC;

    sqr.real += c.real;
    sqr.imag += c.imag;

    return sqr;
}


long mag2(struct Complex z)
{
    return (z.imag * z.imag + z.real * z.real) / PREC;
}