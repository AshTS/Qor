#ifndef _LIBBMP
#define _LIBBMP

typedef struct BitmapHeader
{
    // Common Header
    char magic0;
    char magic1;

    int file_size;
    int res_0;
    int pixel_data_offset;

    // Just the windows header
    int header_size;
    int width;
    int height;
    short color_panes;
    short bits_per_pixel;
    int compression_method;
    int image_data_size;
    int horiz_res;
    int vert_res;
    int color_count;
    int important_colors;
} __attribute__((packed)) BitmapHeader;

#endif // _LIBBMP