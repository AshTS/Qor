#include "printf.h"
#include "signals.h"

int main(int argc, char** argv)
{
    int signal = 9;
    int pid;

    if (argc < 1)
    {
        eprintf("kill requires atleast one argument\n");
        return -1;
    }

    for (int i = 1; i < argc; i++)
    {
        char* s = argv[i];

        if (s[0] == '-')
        {
            int v = 0;
            int j = 1;

            while (s[j])
            {
                v *= 10;
                v += s[j++] - '0';
            }

            signal = v;
        }
        else
        {
            int v = 0;
            int j = 0;

            while (s[j])
            {
                v *= 10;
                v += s[j++] - '0';
            }

            pid = v;
        }
    }

    printf("Sending signal %i to PID %i\n", signal, pid);

    kill(pid, signal);

    return 0;
}