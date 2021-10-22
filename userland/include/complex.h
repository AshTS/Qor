#ifndef _COMPLEX_H
#define _COMPLEX_H

struct Complex
{
    float real;
    float imag;
};

struct Complex cadd(struct Complex a, struct Complex b)
{
    return (struct Complex){.real = a.real + b.real, .imag = a.imag + b.imag};
}

struct Complex csub(struct Complex a, struct Complex b)
{
    return (struct Complex){.real = a.real - b.real, .imag = a.imag - b.imag};
}

struct Complex cmult(struct Complex a, struct Complex b)
{
    return (struct Complex){.real = a.real * b.real - a.imag * b.imag, .imag = a.real * b.imag + a.imag * b.real};
}

struct Complex cconj(struct Complex a)
{
    return (struct Complex){.real = a.real, .imag = -a.imag};
}

float cabs2(struct Complex a)
{
    return a.real * a.real + a.imag * a.imag;
}

struct Complex cdiv(struct Complex a, struct Complex b)
{
    float v = cabs2(b);

    a = cmult(a, cconj(b));

    a.real /= v;
    a.imag /= v;

    return a;
}

struct Complex cscale(struct Complex a, float scale)
{
    a.imag *= scale;
    a.real *= scale;

    return a;
}

#endif // _COMPLEX_H