#ifndef _LIBGRAPHICS_H
#define _LIBGRAPHICS_H

#define LIBGRAPHICS_UNINITIALIZED_FRAMEBUFFER 1
#define LIBGRAPHICS_UNMAPPED_FRAMEBUFFER 2
#define LIBGRAPHICS_UNABLE_TO_OPEN_FRAMEBUFFER 3
#define LIBGRAPHICS_UNABLE_TO_MAP_FRAMEBUFFER 4

extern int LIBGRAPHICS_ERROR;

struct Pixel
{
    char r;
    char g;
    char b;
    char a;
};

int init_framebuffer();
int close_framebuffer();

struct Pixel* get_framebuffer();

int compute_location(int x, int y);

#endif // _LIBGRAPHICS_H