#include "printf.h"
#include "libgraphics.h"
#include "complex.h"

#define CENTER (struct Complex){.real = -0.75, .imag = 0.0}

void libgraphics_error();
struct Pixel shader(int x, int y, int m_or_n);

struct Complex point_to_complex(int x, int y, struct Complex center);

struct Pixel mandelbrot(struct Complex c);
struct Pixel newton(struct Complex c, 
                    struct Complex (func)(struct Complex), 
                    struct Complex (deriv)(struct Complex), 
                    struct Complex zeros[], int zero_count,
                    int iterations);

void help();

int main(int argc, char** argv)
{
    if (argc < 2 || argv[1][0] != '-' || (argv[1][1] != 'm' && argv[1][1] != 'n') || argv[1][2] != 0)
    {
        help();

        return -1;
    }

    int m_or_n = argv[1][1] == 'm';

    // Attempt to initialize the framebuffer, otherwise display the error and
    // return
    if (init_framebuffer() < 0)
    {
        libgraphics_error();
        return -1;
    }

    // Attempt to get the framebuffer
    struct Pixel* framebuffer = get_framebuffer();

    // Make sure the framebuffer exists, otherwise display the error and return
    if (framebuffer == 0)
    {
        libgraphics_error();
        return -1;
    }

    // Loop over every pixel and request its color from the 'shader'
    for (int x = 0; x < 640; x++)
    {
        for (int y = 0; y < 480; y++)
        {
            framebuffer[compute_location(x, y)] = shader(x, y, m_or_n);
        }
    }

    // Attempt to close the framebuffer, otherwise display the error and return
    if (close_framebuffer() < 0)
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
    return sub(mult(z, mult(z, z)), (struct Complex){.real = 1.0, .imag = 0.0});
}

// f'(z) = 3z^2
struct Complex d(struct Complex z)
{
    return scale(mult(z, z), 3);
}

// Shader function, maps an (x, y) pair to a color
struct Pixel shader(int x, int y, int m_or_n)
{
    struct Complex c = point_to_complex(x, y, CENTER);
    
    if (m_or_n)
    {
        return mandelbrot(c);
    }
    else
    {
        float sqrt3d2 = 0.86602540378;

        struct Complex zeros[3] = {(struct Complex){.real = 1.0, .imag = 0.0},
                                (struct Complex){.real = -0.5, .imag = sqrt3d2},
                                (struct Complex){.real = -0.5, .imag = -sqrt3d2}};

        return newton(c, f, d, zeros, 3, 10);
    }
}

// Convert an (x, y) point to a complex number for the shader
struct Complex point_to_complex(int x, int y, struct Complex center)
{
    return add(center, (struct Complex){.real = (float)(x - 320) / 200.0, .imag = (float)(y - 240) / 200.0});
}

// Mandelbrot set shader
struct Pixel mandelbrot(struct Complex c)
{
    struct Complex z = (struct Complex){.real = 0.0, .imag = 0.0};

    char val = 255;

    while (val > 0)
    {
        if (abs2(z) > 4.0)
        {
            break;
        }

        z = add(mult(z, z), c);
        val -= 1;
    }

    return (struct Pixel){.r = val, .g = val, .b = val, .a = 255};
}

// Newton Fractal
struct Pixel newton(struct Complex c, 
                    struct Complex (func)(struct Complex), 
                    struct Complex (deriv)(struct Complex), 
                    struct Complex zeros[], int zero_count,
                    int iterations)
{
    static struct Pixel colors[6] = {(struct Pixel){.r = 255, .g = 128, .b = 128, .a = 255}, 
                                     (struct Pixel){.r = 128, .g = 255, .b = 128, .a = 255}, 
                                     (struct Pixel){.r = 128, .g = 128, .b = 255, .a = 255}, 
                                     (struct Pixel){.r = 255, .g = 255, .b = 128, .a = 255}, 
                                     (struct Pixel){.r = 128, .g = 255, .b = 255, .a = 255}, 
                                     (struct Pixel){.r = 255, .g = 128, .b = 255, .a = 255}};

    for (int i = 0; i < iterations; i++)
    {
        c = sub(c, div(func(c), deriv(c)));
    }

    float best_dist = abs2(sub(c, zeros[0]));
    int best = 0;

    for (int i = 1; i < zero_count; i++)
    {
        float dist = abs2(sub(c, zeros[i]));

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