#include "libc/string.h"
#include "libc/assert.h"

#include "libgraphics.h"

#define MAX(a, b) ((a) < (b) ? (b) : (a))
#define MIN(a, b) ((a) > (b) ? (b) : (a))

#define PI 3.1415926535897932384626433832795028841971693993751058209749445923078164062862089986280348253421170679
#define PI2 PI / 2.0
#define PI4 PI / 4.0

#define ITER(i) i

float scale = 100.0;
float offset = 0.0;

struct Pixel grid_shader(int x, int y);
float function(float x);
float function2(float x);
int float_to_int_y(float y);

void plot_x(int i, struct Pixel* buffer, float f(float), struct Pixel color);

int main(int argc, char** argv)
{
    while (1)
    {
        init_framebuffer();

        // run_individual_shader(grid_shader);

        struct Pixel* buffer = get_framebuffer();

        if (buffer == 0)
        {
            return -1;
        }

        for (int i = 0; i < 640; i++)
        {
            plot_x(i, buffer, function, COLOR_RED);
            // plot_x(i, buffer, function2, COLOR_BLUE);
        }

        close_framebuffer();
        
        offset += 0.05;
    }
}

void plot_x(int i, struct Pixel* buffer, float f(float), struct Pixel color)
{
    int y0 = float_to_int_y(f(((float)i - 0.5 - 320.0) / scale));
    int y1 = float_to_int_y(f(((float)i + 0.5 - 320.0) / scale));

    for (int y = MIN(y0, y1); y <= MAX(y0, y1); y++)
    {
        if (y < 0 || y >= 480)
        {
            continue;
        }

        buffer[compute_location(i, y)] = color;
    }
}

struct Pixel grid_shader(int x, int y)
{
    x -= 320;
    y -= 240;

    if (x % (int)scale == 0 || y % (int)scale == 0)
    {
        return COLOR_WHITE;
    }
    else if (x % ((int)scale / 4) == 0 || y % ((int)scale / 4) == 0)
    {
        return COLOR_GREY;
    }

    return COLOR_BLACK;
}

double cos(double x)
{
    if (x < 0.0)
    {
        return cos(-x);
    }

    if (x > PI)
    {
        int n = x / PI;

        if (n % 2 == 0)
        {
            return cos(x - (double)n*PI);
        }
        else
        {
            return -cos(x - (double)n*PI);
        }
    }

    if (x > PI2)
    {
        return -cos(PI - x);
    }

    int iterations = 9;

    if (x < 1.4e-5)
    {
        iterations = 1;
    }
    else if (x < 0.00026)
    {
        iterations = 2;
    }
    else if (x < 0.0247)
    {
        iterations = 3;
    }
    else if (x < 0.093)
    {
        iterations = 4;
    }
    else if (x < 0.22)
    {
        iterations = 5;
    }
    else if (x < 0.417)
    {
        iterations = 6;
    }
    else if (x < 0.684)
    {
        iterations = 7;
    }
    else if (x < 1.36)
    {
        iterations = 8;
    }

    double result = 1.0;
    double running = x * x / 2.0;

    for (int i = 0; i < ITER(iterations); i++)
    {
        if (i % 2 == 0)
        {
            result -= running;
        }
        else
        {
            result += running;
        }

        running *= x * x;
        running /= (double)(2 * (i + 1) + 1);
        running /= (double)(2 * (i + 1) + 2);
    }

    return result;
}

double sin(double x)
{
    if (x < 0.0)
    {
        return -sin(-x);
    }

    if (x > PI)
    {
        int n = x / PI;

        if (n % 2 == 0)
        {
            return sin(x - (double)n*PI);
        }
        else
        {
            return -sin(x-(double)n*PI);
        }
    }

    if (x > PI2)
    {
        return sin(PI - x);
    }

    int iterations = 11;

    if (x < 2.6e-8)
    {
        iterations = 1;
    }
    else if (x < 0.0003653)
    {
        iterations = 2;
    }
    else if (x < 0.0098)
    {
        iterations = 3;
    }
    else if (x < 0.05)
    {
        iterations = 4;
    }
    else if (x < 0.15)
    {
        iterations = 5;
    }
    else if (x < 0.315)
    {
        iterations = 6;
    }
    else if (x < 0.55) 
    {
        iterations = 7;
    }
    else if (x < 0.82)
    {
        iterations = 8;
    }
    else if (x < 1.1985)
    {
        iterations = 9;
    }
    else if (x < 1.56)
    {
        iterations = 10;
    }

    double result = 0.0;
    double running = x;

    for (int i = 0; i < ITER(iterations); i++)
    {
        if (i % 2 == 0)
        {
            result += running;
        }
        else
        {
            result -= running;
        }

        running *= x * x;
        running /= (double)(2 * (i + 1));
        running /= (double)(2 * (i + 1) + 1);
    }

    return result;
}

float function(float x)
{
    // return sin(2.0*x * PI) + cos(2.0 * x * PI + offset) + sin(3.0 * x * PI);
    // return (x - 1.0 / (1.0 + offset)) * (x + offset) * (x - offset);

    //return sin(cos(x + offset) * PI + cos(offset*x));

    return sin(3.0 * offset);
}



int float_to_int_y(float y)
{
    return 240 - (y * scale);
}