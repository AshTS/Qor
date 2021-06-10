#include <malloc.h>
#include <printf.h>
#include <string.h>
#include <syscalls.h>

#include "parser.h"
#include "interpreter.h"

int input(char buffer[], int size)
{
    int count;
    do
    {
        count = read(0, buffer, size - 1);
    }
    while (count == 0);

    buffer[count - 1] = 0;

    return count;
}

void load_file(char* name, InterpreterState* state)
{
    int file = open(name, 0);

    if (file < 0)
    {
        printf("Unable to open file `%s`\n", name);
        return;
    }

    // Input Buffer
    char input_buffer[4096];

    int size = read(file, input_buffer, 4095);
    close(file);
    input_buffer[size] = 0;

    int index = 0;
    int line_start = 0;

    while (input_buffer[line_start] != 0)
    {
        index = line_start;

        while (input_buffer[index] != 0)
        {
            index += 1;

            if (input_buffer[index - 1] == '\n')
            {
                break;
            }
        }

        if (index == line_start || (input_buffer[index - 1] == '\n' && index == line_start + 1))
        {
            line_start = index;
            continue;
        }

        if (input_buffer[index - 1] == '\n')
        {
            input_buffer[index - 1] = 0;
        }

        char* this_buffer = input_buffer + line_start;     

        // Space for the tokens
        unsigned int token_count_max = 32;
        Token** token_space = malloc(sizeof(Token*) * token_count_max);

        // Display all of the tokens
        unsigned int token_index = 0;
        Token* token;
        while(1)
        {
            token = parse_next_token(&this_buffer);
            token_space[token_index++] = token;

            if (token_index == token_count_max)
            {
                printf("Too many tokens!\n");
                return;
            }

            if (token == 0)
            {
                break;
            }
        }

        interpret(state, token_space);

        for (int i = 0; i + 1 < token_index; i++)
        {
            drop_token(token_space[i]);
        }

        free(token_space);

        line_start = index;
    }

    printf("Loaded program from %s\n", name);
}

int main(int argc, char** argv)
{
    printf("QORBasic\n");

    InterpreterState* state = construct_interpreter();

    if (argc > 1)
    {
        load_file(argv[1], state);
    }

    // Input Buffer
    char input_buffer[256];

    while(1)
    {
        printf("> ");
        
        input(input_buffer, 256);

        // Check for quit
        if (strcmp(input_buffer, "quit") == 0)
        {
            break;
        }

        if (input_buffer[0] == 'l' &&
            input_buffer[1] == 'o' &&
            input_buffer[2] == 'a' &&
            input_buffer[3] == 'd' &&
            input_buffer[4] == ' ')
        {
            load_file(&input_buffer[5], state);
            continue;
        }

        if (strlen(input_buffer) == 0)
        {
            continue;
        }

        char* this_buffer = input_buffer;

        // Space for the tokens
        unsigned int token_count_max = 32;
        Token** token_space = malloc(sizeof(Token*) * token_count_max);

        // Display all of the tokens
        unsigned int token_index = 0;
        Token* token;
        while(1)
        {
            token = parse_next_token(&this_buffer);
            token_space[token_index++] = token;

            if (token_index == token_count_max)
            {
                printf("Too many tokens!\n");
                return -1;
            }

            /*
            if (token != 0)
            {
                display_token(token);
            }*/

            if (token == 0)
            {
                break;
            }
        }

        interpret(state, token_space);

        for (int i = 0; i + 1 < token_index; i++)
        {
            drop_token(token_space[i]);
        }

        free(token_space);
    }

    drop_interpreter(state);

    dump();

    return 0;
}
