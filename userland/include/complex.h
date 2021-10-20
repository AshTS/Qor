#ifndef _COMPLEX_H
#define _COMPLEX_H

struct Complex
{
    float real;
    float imag;
};

struct Complex add(struct Complex a, struct Complex b)
{
    return (struct Complex){.real = a.real + b.real, .imag = a.imag + b.imag};
}

struct Complex sub(struct Complex a, struct Complex b)
{
    return (struct Complex){.real = a.real - b.real, .imag = a.imag - b.imag};
}

struct Complex mult(struct Complex a, struct Complex b)
{
    return (struct Complex){.real = a.real * b.real - a.imag * b.imag, .imag = a.real * b.imag + a.imag * b.real};
}

struct Complex conj(struct Complex a)
{
    return (struct Complex){.real = a.real, .imag = -a.imag};
}

float abs2(struct Complex a)
{
    return a.real * a.real + a.imag * a.imag;
}

struct Complex div(struct Complex a, struct Complex b)
{
    float v = abs2(b);

    a = mult(a, conj(b));

    a.real /= v;
    a.imag /= v;

    return a;
}

struct Complex scale(struct Complex a, float scale)
{
    a.imag *= scale;
    a.real *= scale;

    return a;
}

#endif // _COMPLEX_H