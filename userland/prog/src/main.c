#include "printf.h"
#include "libgraphics.h"

int main(int argc, char** argv)
{
    printf("Libgraphics Test\n");
    if (init_framebuffer() < 0)
    {
        eprintf("ErrorNo: %i\n", LIBGRAPHICS_ERROR);
        return -1;
    }

    struct Pixel* framebuffer = get_framebuffer();

    if (framebuffer == 0)
    {
        eprintf("ErrorNo: %i\n", LIBGRAPHICS_ERROR);
        return -1;
    }

    struct Pixel color = {.r = 255, .g = 128, .b = 128, .a = 255};

    for (int x = 50; x < 100; x++)
    {
        for (int y = 100; y < 150; y++)
        {
            framebuffer[compute_location(x, y)] = color;
        }
    }

    if (close_framebuffer() < 0)
    {
        eprintf("ErrorNo: %i\n", LIBGRAPHICS_ERROR);
        return -1;
    }
    return 0;
}
