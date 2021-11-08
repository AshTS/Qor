#ifndef _LIBGRAPHICS_H
#define _LIBGRAPHICS_H

#define LIBGRAPHICS_UNINITIALIZED_FRAMEBUFFER 1
#define LIBGRAPHICS_UNMAPPED_FRAMEBUFFER 2
#define LIBGRAPHICS_UNABLE_TO_OPEN_FRAMEBUFFER 3
#define LIBGRAPHICS_UNABLE_TO_MAP_FRAMEBUFFER 4

#define COLOR_BLACK (struct Pixel){.r=0, .g=0, .b=0, .a=255}
#define COLOR_WHITE (struct Pixel){.r=255, .g=255, .b=255, .a=255}
#define COLOR_GREY (struct Pixel){.r=128, .g=128, .b=128, .a=255}
#define COLOR_RED (struct Pixel){.r=255, .g=0, .b=0, .a=255}
#define COLOR_GREEN (struct Pixel){.r=0, .g=255, .b=0, .a=255}
#define COLOR_BLUE (struct Pixel){.r=0, .g=0, .b=255, .a=255}
#define COLOR_MAGENTA (struct Pixel){.r=255, .g=0, .b=255, .a=255}
#define COLOR_YELLOW (struct Pixel){.r=255, .g=255, .b=0, .a=255}
#define COLOR_CYAN (struct Pixel){.r=0, .g=255, .b=255, .a=255}
#define COLOR_LIGHT_RED (struct Pixel){.r=255, .g=128, .b=128, .a=255}
#define COLOR_LIGHT_GREEN (struct Pixel){.r=128, .g=255, .b=128, .a=255}
#define COLOR_LIGHT_BLUE (struct Pixel){.r=128, .g=128, .b=255, .a=255}
#define COLOR_LIGHT_MAGENTA (struct Pixel){.r=255, .g=128, .b=255, .a=255}
#define COLOR_LIGHT_YELLOW (struct Pixel){.r=255, .g=255, .b=128, .a=255}
#define COLOR_LIGHT_CYAN (struct Pixel){.r=128, .g=255, .b=255, .a=255}


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

int run_shader(struct Pixel (shader)(int, int));
int run_individual_shader(struct Pixel (shader)(int, int));
int flush_framebuffer();

#endif // _LIBGRAPHICS_H