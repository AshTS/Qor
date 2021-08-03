#include "printf.h"
#include "syscalls.h"
#include "string.h"
#include "malloc.h"

#define FBIOGET_VSCREENINFO	0x4600
#define FBIOPUT_VSCREENINFO	0x4601
#define FBIOGET_FSCREENINFO	0x4602

struct fb_bitfield
{
	unsigned int offset;			/* beginning of bitfield	*/
	unsigned int length;			/* length of bitfield		*/
	unsigned int msb_right;		/* != 0 : Most significant bit is */ 
					/* right */ 
};

struct fb_fix_screeninfo
{
      char id[16];                    /* identification string eg "TT Builtin" */
      unsigned long smem_start;       /* Start of frame buffer mem */
                                      /* (physical address) */
      unsigned int smem_len;                 /* Length of frame buffer mem */
      unsigned int type;                     /* see FB_TYPE_*                */
      unsigned int type_aux;                 /* Interleave for interleaved Planes */
      unsigned int visual;                   /* see FB_VISUAL_*              */
      unsigned short xpanstep;                 /* zero if no hardware panning  */
      unsigned short ypanstep;                 /* zero if no hardware panning  */
      unsigned short ywrapstep;                /* zero if no hardware ywrap    */
      unsigned int line_length;              /* length of a line in bytes    */
      unsigned long mmio_start;       /* Start of Memory Mapped I/O   */
                                      /* (physical address) */
      unsigned int mmio_len;                 /* Length of Memory Mapped I/O  */
      unsigned int accel;                    /* Indicate to driver which     */
                                      /*  specific chip/card we have  */
      unsigned short capabilities;             /* see FB_CAP_*                 */
      unsigned short reserved[2];              /* Reserved for future compatibility */
};

struct fb_var_screeninfo
{
	unsigned int xres;			/* visible resolution		*/
	unsigned int yres;
	unsigned int xres_virtual;		/* virtual resolution		*/
	unsigned int yres_virtual;
	unsigned int xoffset;			/* offset from virtual to visible */
	unsigned int yoffset;			/* resolution			*/

	unsigned int bits_per_pixel;		/* guess what			*/
	unsigned int grayscale;		/* 0 = color, 1 = grayscale,	*/
					/* >1 = FOURCC			*/
	struct fb_bitfield red;		/* bitfield in fb mem if true color, */
	struct fb_bitfield green;	/* else only length is significant */
	struct fb_bitfield blue;
	struct fb_bitfield transp;	/* transparency			*/	

	unsigned int nonstd;			/* != 0 Non standard pixel format */

	unsigned int activate;			/* see FB_ACTIVATE_*		*/

	unsigned int height;			/* height of picture in mm    */
	unsigned int width;			/* width of picture in mm     */

	unsigned int accel_flags;		/* (OBSOLETE) see fb_info.flags */

	/* Timing: All values in pixclocks, except pixclock (of course) */
	unsigned int pixclock;			/* pixel clock in ps (pico seconds) */
	unsigned int left_margin;		/* time from sync to picture	*/
	unsigned int right_margin;		/* time from picture to sync	*/
	unsigned int upper_margin;		/* time from sync to picture	*/
	unsigned int lower_margin;
	unsigned int hsync_len;		/* length of horizontal sync	*/
	unsigned int vsync_len;		/* length of vertical sync	*/
	unsigned int sync;			/* see FB_SYNC_*		*/
	unsigned int vmode;			/* see FB_VMODE_*		*/
	unsigned int rotate;			/* angle we rotate counter clockwise */
	unsigned int colorspace;		/* colorspace for FOURCC-based modes */
	unsigned int reserved[4];		/* Reserved for future compatibility */
};

int main(int argc, char** argv)
{
    int fd = open("/dev/fb0", O_WRONLY);

    if (fd < 0)
    {
        printf("Unable to open /dev/fb0\n");
        return -1;
    }

    struct fb_var_screeninfo var_data;
    struct fb_fix_screeninfo fix_data;

    ioctl(fd, FBIOGET_VSCREENINFO, &var_data);
    ioctl(fd, FBIOGET_FSCREENINFO, &fix_data);

	printf("Device Name: `%s`\n", fix_data.id);
    printf("Display is %i x %i\n", var_data.xres, var_data.yres);

    close(fd);
}
