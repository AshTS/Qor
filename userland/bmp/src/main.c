#include "printf.h"
#include "syscalls.h"
#include "malloc.h"

#include "libbmp/libbmp.h"

struct Pixel
{
    char r;
    char g;
    char b;
    char a;
};

int main(int argc, char** argv)
{
    if (argc < 2)
    {
        printf("%s required atleast one argument, the file to display\n", argv[0]);
        return -1;
    }

    const char* path = argv[1];

    printf("Open File `%s`\n", path);

    int fd = open(path, O_RDONLY);

    if (fd == -1)
    {
        printf("Unable to open file `%s`\n", path);
        return -1;
    }

    BitmapHeader header;

    read(fd, &header, sizeof(BitmapHeader));

    printf("Image is %i x %i\n", header.width, header.height);

    struct Pixel* buffer = malloc(header.width * header.height * sizeof(struct Pixel));

    lseek(fd, header.pixel_data_offset, SEEK_SET);

    char cur[3];

    for (int y = header.height - 1; y >= 0; y--)
    {
        int count_read = 0;

        for (int x = 0; x < header.width; x++)
        {
            count_read += 3;
            int amt = read(fd, cur, 3);

            buffer[header.width * y + x] = (struct Pixel){.r = cur[2], .g = cur[1], .b = cur[0], .a = 255};
        }

        lseek(fd, (4 - (count_read % 4)) % 4, SEEK_CUR);
    }

    close(fd);

    int fb_fd = open("/dev/fb0", O_WRONLY);

    if (fb_fd == -1)
    {
        printf("Unable to open /dev/fb0\n");
        return -1;
    }

    for (int y = 0; y < header.height; y++)
    {
        lseek(fb_fd, 4 * 640 * y, SEEK_SET);
        write(fb_fd, &buffer[y * header.width], header.width * sizeof(struct Pixel));
    }

    close(fb_fd);

    free(buffer);

    return 0;
}
