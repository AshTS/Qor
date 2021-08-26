#include "printf.h"
#include "syscalls.h"
#include "types.h"
#include "malloc.h"

#define SCALE 1000
#define RESOLUTION 100000

struct Pixel
{
    char r;
    char g;
    char b;
    char a;
};


void bezier_draw(struct Pixel* fb, struct Pixel color, long* xs, long* ys, int count);

long bezier_lerp(long value, long* array, int count);

void display_bezier(long* xs, long* ys, int count);

int main(int argc, char** argv)
{
    long* xs = malloc(2 * sizeof(long));
    long* ys = malloc(2 * sizeof(long));
    int fd = 0;
    int count = 0;
    int num_alloc = 2;

    if (argc > 1)
    {
        if (argc == 2)
        {
            fd = open(argv[1], O_RDONLY);

            if (fd < 0)
            {
                eprintf("Unable to open file `%s`\n", argv[1]);
            }
        }
        else if (argc % 2 == 0)
        {
            eprintf("Specifying points on the command line requires an even number of arguments\n");   
        }
        else
        {
            for (int i = 1; i < argc; i++)
            {
                if (count == num_alloc)
                {
                    num_alloc *= 2;

                    long* old_x = xs;
                    long* old_y = ys;

                    xs = malloc(num_alloc * sizeof(long));
                    ys = malloc(num_alloc * sizeof(long));

                    for (int i = 0; i < count; i++)
                    {
                        xs[i] = old_x[i];
                        ys[i] = old_y[i];
                    }

                    free(old_x);
                    free(old_y);
                }

                long v = 0;
                char* c = &argv[i][0];

                while (*c != 0)
                {
                    v *= 10;
                    v += ((long)(*c - '0'));

                    c++;
                }

                printf("%i\n", v);

                if (i % 2 == 1)
                {
                    xs[(i - 1) / 2] = v;
                }
                else
                {
                    ys[(i - 1) / 2] = v;
                    count++;
                }
            }
        }
    }
    else
    {
        eprintf("bezier does not support file reading yet\n");
        return -1;
    }

    display_bezier(xs, ys, count);

    return 0;
}

void display_bezier(long* xs, long* ys, int count)
{
    int fb_fd = open("/dev/fb0", O_WRONLY);

    if (fb_fd == -1)
    {
        eprintf("Unable to open /dev/fb0\n");
        return;
    }

    struct Pixel* framebuffer = mmap(0, 640 * 480 * 4, PROT_READ | PROT_WRITE, 0, fb_fd, 0);

    struct Pixel color;

    color.r = 255;
    color.g = 255;
    color.b = 255;
    color.r = 255;

    bezier_draw(framebuffer, color, xs, ys, count);
    
    munmap(framebuffer, 640 * 480 * 4);

    close(fb_fd);
}

// Lerp with a value ranging from 0 to SCALE
long lerp(long value, long a, long b)
{
    return a + ((b - a) * value) / SCALE;
}

long bezier_lerp(long value, long* array, int count)
{
    if (count == 2)
    {
        return lerp(value, array[0], array[1]);
    }
    else if (count == 1)
    {
        return array[0];
    }
    
    long* data = malloc((count - 1) * sizeof(long));

    for (int i = 0; i < count - 1; i++)
    {
        data[i] = lerp(value, array[i], array[i + 1]);
    }

    long result = bezier_lerp(value, data, count - 1);

    free(data);

    return result;
}

void bezier_draw(struct Pixel* fb, struct Pixel color, long* xs, long* ys, int count)
{
    for (int i = 0; i < RESOLUTION; i++)
    {
        long v = i * SCALE / RESOLUTION;

        int x = (int)bezier_lerp(v, xs, count);
        int y = (int)bezier_lerp(v, ys, count);

        fb[x + y * 640] = color;
    }
}