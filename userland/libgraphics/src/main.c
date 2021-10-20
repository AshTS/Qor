#include "libgraphics.h"
#include "syscalls.h"

static int FRAME_BUFFER_FD = -1;
static struct Pixel* frame_buffer = 0;

int LIBGRAPHICS_ERROR = 0;

int verify_framebuffer()
{
    if (FRAME_BUFFER_FD < 0)
    {
        LIBGRAPHICS_ERROR = LIBGRAPHICS_UNINITIALIZED_FRAMEBUFFER;
        return -1;
    }

    if (frame_buffer == 0)
    {
        LIBGRAPHICS_ERROR = LIBGRAPHICS_UNMAPPED_FRAMEBUFFER;
        return -1;
    }

    return 0;
}

int init_framebuffer()
{
    FRAME_BUFFER_FD = open("/dev/fb0", O_WRONLY);

    if (FRAME_BUFFER_FD < 0)
    {
        LIBGRAPHICS_ERROR = LIBGRAPHICS_UNABLE_TO_OPEN_FRAMEBUFFER;
        return -1;
    }

    frame_buffer = mmap(0, 640 * 480 * 4, PROT_READ | PROT_WRITE, 0, FRAME_BUFFER_FD, 0);

    if (frame_buffer == 0)
    {
        LIBGRAPHICS_ERROR = LIBGRAPHICS_UNABLE_TO_MAP_FRAMEBUFFER;
        return -1;
    }

    return 0;
}

int close_framebuffer()
{
    if (verify_framebuffer() < 0)
    {
        return -1;
    }

    munmap(frame_buffer, 640 * 480 * 4);

    close(FRAME_BUFFER_FD);

    FRAME_BUFFER_FD = -1;
    frame_buffer = 0;

    return 0;
}

struct Pixel* get_framebuffer()
{
    if (verify_framebuffer() < 0)
    {
        return 0;
    }

    return frame_buffer;
}

int compute_location(int x, int y)
{
    return 640 * y + x;
}