#include "printf.h"
#include "syscalls.h"
#include "string.h"
#include "malloc.h"

#define POS_TO_INDEX(x, y) ((4 * (x)) + (y) * 2560)

typedef struct
{
    char red;
    char green;
    char blue;
    char alpha;
} Pixel;

Pixel new_pixel(char r, char g, char b)
{
    Pixel p;

    p.red = r;
    p.green = g;
    p.blue = b;
    p.alpha = 255;

    return p;
}

void draw_box(int fd, int x, int y, int width, int height, Pixel color);

int start_context();
void end_context(int);

int main(int argc, char** argv)
{
    int fd = open("/dev/tty0", O_WRONLY);

    write(fd, "Hi", 2);

    close(fd);

    /*
    int fd = start_context();

    draw_box(fd, 25, 25, 25, 25, new_pixel(255, 200, 255));

    end_context(fd);*/
}

void draw_box(int fd, int x, int y, int width, int height, Pixel color)
{
    // Initialize the single line
    Pixel* line = malloc(4 * width);

    for (int i = 0; i < width; i++)
    {
        line[i] = color;
    }

    for (int row = 0; row < height; row++)
    {
        int pos = POS_TO_INDEX(x, y + row);

        lseek(fd, POS_TO_INDEX(x, y + row), SEEK_SET);
        write(fd, line, 4 * width);
    }

    free(line);
}

int start_context()
{
    int fd = open("/dev/fb0", O_WRONLY | O_TRUNC);

    if (fd == -1)
    {
        printf("Unable to open `/dev/fb0`\n");
        exit(-1);
    }

    return fd;
}

void end_context(int fd)
{
    close(fd);
}