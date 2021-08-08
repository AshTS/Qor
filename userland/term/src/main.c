#include "syscalls.h"
#include "printf.h"
#include "string.h"
#include "signals.h"

#define ESC 27
#define TERMINATE 3

#define COL0 0
#define COL1 35

#define WIDTH 33
#define HEIGHT 30

void handle_side(int disp, int amt, char* buffer, int* cursor, int col, int color)
{
    fprintf(disp, "\x1B[%im\x1B[%i;%iH", color, col + cursor[0], cursor[1]);

    char has_jumped = 0;

    for (int i = 0; i < amt; i++)
    {
        char c = buffer[i];

        if (c == '\n')
        {
            while (cursor[0] <= WIDTH)
            {
                write(disp, " ", 1);
                cursor[0]++;
            }

            cursor[0] = 1;
            cursor[1] += 1;

            has_jumped = 1;
        }
        else
        {
            if (has_jumped)
            {
                fprintf(disp, "\x1B[%i;%iH", col + cursor[0], cursor[1]);
            }

            write(disp, &c, 1);

            has_jumped = 0;

            cursor[0] += 1;
        }

        if (cursor[0] > WIDTH)
        {
            cursor[0] = 1;
            cursor[1] += 1;

            has_jumped = 1;
        }

        cursor[1] %= HEIGHT;
    }
}

void single_listener(char* dev, int f0)
{
    int disp = open(dev, O_WRONLY);

    int cursor0[] = {1, 1};
    int cursor1[] = {1, 1};

    char buffer[512];

    while (1)
    {
        int amt;
        amt = read(f0, buffer, 512);

        if (amt > 0)
        {
            write(disp, buffer, amt);

            close(disp);
            disp = open(dev, O_WRONLY);
        }
    }
}

void splitter(int f0, int f1)
{
    int disp = open("/dev/disp", O_WRONLY);

    write(disp, "\x1B[0m", 4);

    // Render a bar down the center of the screen
    for (int y = 0; y < 30; y++)
    {
        char buffer[64];

        sprintf(buffer, "\x1B[%i;%iH\xBA", COL1 - 1, y + 1);

        int length = strlen(buffer);

        write(disp, buffer, length);
    }

    close(disp);

    int cursor0[] = {1, 1};
    int cursor1[] = {1, 1};

    char buffer[512];

    disp = open("/dev/disp", O_WRONLY);

    while (1)
    {
        int amt;
        amt = read(f0, buffer, 512);

        if (amt > 0)
        {
            handle_side(disp, amt, buffer, cursor0, COL0, 0);

            close(disp);
            disp = open("/dev/disp", O_WRONLY);
        }

        amt = read(f1, buffer, 512);

        if (amt > 0)
        {
            handle_side(disp, amt, buffer, cursor1, COL1, 0);

            close(disp);
            disp = open("/dev/disp", O_WRONLY);
        }
    }

    
}

int start_shell()
{
    int r = fork();
    if (r == 0)
    {
        const char* name = "/bin/shell";
        const char* argv[2];

        argv[0] = name;
        argv[1] = 0;

        const char* envp[1];

        envp[0] = 0;

        execve("/bin/shell", argv, envp);
    }

    return r;
}

int redirect_from(int fd)
{
    int fds[2];
    pipe(fds);

    dup2(fds[0], fd);

    return fds[1];
}

int redirect_to(int fd)
{
    int fds[2];
    pipe(fds);

    dup2(fds[1], fd);

    return fds[0];
}

int main(int argc, char** argv)
{
    char* device = "/dev/tty0";
    char run_dual = 0;

    if (argc > 2)
    {
        device = argv[1];
        run_dual = 1;
    }
    else if (argc > 1)
    {
        device = argv[1];
    }

    open("/dev/null", O_RDONLY);
    open(device, O_WRONLY);
    open("/dev/tty0", O_WRONLY);

    int fd = open("/dev/tty0", O_RDONLY);

    int in0, in1, out0, out1, disp0, disp1;

    in0 = redirect_from(0);
    out0 = redirect_to(1);
    disp0 = dup(1);

    int shell_pid = start_shell();

    if (run_dual)
    {
        in1 = redirect_from(0);
        out1 = redirect_to(1);
        disp1 = dup(1);

        start_shell();
    }

    if (fork() == 0)
    {
        if (run_dual)
        {
            splitter(out0, out1);
        }
        else
        {
            single_listener(device, out0);
        }
    }

    char buffer[256];
    int buffer_index = 0;

    char side = 0;

    int in[] = {in0, in1};
    int out[] = {disp0, disp1};

    if (!run_dual)
    {
        in[1] = in[0];
        out[1] = out[0];
    }

    while (1)
    {
        char c; 
        if (read(fd, &c, 1))
        {
            if (c == TERMINATE)
            {
                kill(shell_pid, SIGINT);
                continue;
            }

            if (c == ESC)
            {
                // fprintf(disp, "\x1B[31mESC\x1B[0m\n", c);
                if (run_dual)
                    side = !side;
                continue;
            }

            if (c == 10 || c == 13)
            {
                write(out[side], "\n", 1);
                write(in[side], buffer, buffer_index + 1);
                buffer_index = 0;
            }
            else if (c == 8 || c == 127)
            {
                if (buffer_index > 0)
                {
                    write(out[side], "\x08 \x08", 3);
                    buffer_index--;
                }
            }
            else
            {

                write(out[side], &c, 1);
                
                buffer[buffer_index] = c;
                buffer_index++;

                if (buffer_index >= 256)
                {
                    buffer_index = 255;
                }
            }
        }
    }

    return 0;
}
