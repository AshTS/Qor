#include "printf.h"
#include "libgraphics.h"
#include "complex.h"

#define CENTER (struct Complex){.real = -0.75, .imag = 0.0}

void libgraphics_error();

struct Complex point_to_complex(int x, int y, struct Complex center);

struct Pixel mandelbrot_shader(int x, int y);
struct Pixel newton_shader(int x, int y);

void help();

struct Complex f(struct Complex z);
struct Complex d(struct Complex z);

static struct Complex zeros[3];
static int zero_count;
static int iterations;

int main(int argc, char** argv)
{
    if (argc < 2 || argv[1][0] != '-' || (argv[1][1] != 'm' && argv[1][1] != 'n') || argv[1][2] != 0)
    {
        help();

        return -1;
    }

    int m_or_n = argv[1][1] == 'm';

    int ret_val;


    // Attempt to run the proper shader
    if (m_or_n)
    {
        printf("Mandelbrot\n");
        
        ret_val = run_shader(mandelbrot_shader);
    }
    else
    {
        printf("Newton\n");

        float sqrt3d2 = 0.86602540378;

        zeros[0] = (struct Complex){.real = 1.0, .imag = 0.0};
        zeros[1] = (struct Complex){.real = -0.5, .imag = sqrt3d2};
        zeros[2] = (struct Complex){.real = -0.5, .imag = -sqrt3d2};

        iterations = 6;

        ret_val = run_shader(newton_shader);
    }

    // Display the error and exit if an error was found
    if (ret_val < 0)
    {
        libgraphics_error();
        return -1;
    }

    return 0;
}

// Display the error message from the libgraphics library
void libgraphics_error()
{
    eprintf("libgraphics error: ");

    if (LIBGRAPHICS_ERROR == LIBGRAPHICS_UNABLE_TO_MAP_FRAMEBUFFER)
    {
        eprintf("UNABLE_TO_MAP_FRAMEBUFFER\n");
    }
    else if (LIBGRAPHICS_ERROR == LIBGRAPHICS_UNABLE_TO_OPEN_FRAMEBUFFER)
    {
        eprintf("UNABLE_TO_OPEN_FRAMEBUFFER\n");
    }
    else if (LIBGRAPHICS_ERROR == LIBGRAPHICS_UNINITIALIZED_FRAMEBUFFER)
    {
        eprintf("UNINITIALIZED_FRAMEBUFFER\n");
    }
    else if (LIBGRAPHICS_ERROR == LIBGRAPHICS_UNMAPPED_FRAMEBUFFER)
    {
        eprintf("UNMAPPED_FRAMEBUFFER\n");
    }
}

// f(z) = z^3 - 1
struct Complex f(struct Complex z)
{
    return csub(cmult(z, cmult(z, z)), (struct Complex){.real = 1.0, .imag = 0.0});
}

// f'(z) = 3z^2
struct Complex d(struct Complex z)
{
    return cscale(cmult(z, z), 3);
}


// Convert an (x, y) point to a complex number for the shader
struct Complex point_to_complex(int x, int y, struct Complex center)
{
    return cadd(center, (struct Complex){.real = (float)(x - 320) / 200.0, .imag = (float)(y - 240) / 200.0});
}

// Mandelbrot set shader
struct Pixel mandelbrot_shader(int x, int y)
{
    struct Complex c = point_to_complex(x, y, CENTER);
    struct Complex z = (struct Complex){.real = 0.0, .imag = 0.0};

    char val = 255;

    while (val > 0)
    {
        if (cabs2(z) > 4.0)
        {
            break;
        }

        z = cadd(cmult(z, z), c);
        val -= 1;
    }

    return (struct Pixel){.r = val, .g = val, .b = val, .a = 255};
}

// Newton Fractal
struct Pixel newton_shader(int x, int y)
{
    struct Complex c = point_to_complex(x, y, CENTER);
    static struct Pixel colors[6] = {(struct Pixel){.r = 255, .g = 128, .b = 128, .a = 255}, 
                                     (struct Pixel){.r = 128, .g = 255, .b = 128, .a = 255}, 
                                     (struct Pixel){.r = 128, .g = 128, .b = 255, .a = 255}, 
                                     (struct Pixel){.r = 255, .g = 255, .b = 128, .a = 255}, 
                                     (struct Pixel){.r = 128, .g = 255, .b = 255, .a = 255}, 
                                     (struct Pixel){.r = 255, .g = 128, .b = 255, .a = 255}};

    for (int i = 0; i < iterations; i++)
    {
        c = csub(c, cdiv(f(c), d(c)));
    }

    float best_dist = cabs2(csub(c, zeros[0]));
    int best = 0;

    for (int i = 1; i < zero_count; i++)
    {
        float dist = cabs2(csub(c, zeros[i]));

        if (dist < best_dist)
        {
            best_dist = dist;
            best = i;
        }
    }

    return colors[best];
}

// Display the help
void help()
{
    printf("Usage: fractal [OPTIONS]\n  Displays fractals using the libgraphics API\n\n");
    
    printf("  -m       Display the Mandelbrot set\n");
    printf("  -n       Display the Newton fractal for f(z) = z^3 - 1\n");
}