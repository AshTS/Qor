#include "printf.h"
#include "board.h"

struct Result
{
    Move m;
    int score;
};

void* memcpy( void* dest, const void* src, int count )
{
    for (int i = 0; i < count; i++)
    {
        *((char*)dest + i) = *((char*)src + i);
    }

    return dest;
}

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

struct Result minmax(Board* board, int level, char is_red);

int main()
{
    printf("Checkers!\n\n");

    Board* board = alloc_board();

    reset_board(board);

    while (1)
    {
        printf("\nEvaluation: %i\n", evaluate_board(board, 0));
        display_board(board);

        printf("Enter a move: ");

        char buffer[64];
        int length = input(buffer, 63);
        buffer[length] = 0;

        char x0, x1, y0, y1;

        if (buffer[0] >= 'A' && buffer[0] <= 'H')
        {
            x0 = buffer[0] - 'A';
        }
        else if (buffer[0] >= 'a' && buffer[0] <= 'h')
        {
            x0 = buffer[0] - 'a';
        }
        else
        {
            eprintf("Bad Move Pos0, Try Again\n");
            continue;
        }

        if (buffer[1] >= '1' && buffer[1] <= '9')
        {
            y0 = buffer[1] - '1';
        }
        else
        {
            eprintf("Bad Move Pos0, Try Again\n");
            continue;
        }

        if (buffer[3] >= 'A' && buffer[3] <= 'H')
        {
            x1 = buffer[3] - 'A';
        }
        else if (buffer[3] >= 'a' && buffer[3] <= 'h')
        {
            x1 = buffer[3] - 'a';
        }
        else
        {
            eprintf("Bad Move Pos1, Try Again\n");
            continue;
        }

        if (buffer[4] >= '1' && buffer[4] <= '9')
        {
            y1 = buffer[4] - '1';
        }
        else
        {
            eprintf("Bad Move Pos1, Try Again\n");
            continue;
        }

        make_move(board, move(x0, y0, x1, y1));

        Move next_move = minmax(board, 5, 1).m;
        make_move(board, next_move);
    }
    

    return 0;
}

struct Result minmax(Board* board, int level, char is_red)
{
    Node* moves = get_moves(board, is_red);
    Node* ptr = moves;

    Move best;
    int score = -10000000;

    while (ptr != 0)
    {
        Move this_move = ptr->move;

        Board* this_board = alloc_board();
        *this_board = *board;

        make_move(this_board, this_move);

        int this_score;

        if (level < 1)
        {
            this_score = evaluate_board(this_board, is_red);
        }
        else
        {
            struct Result result = minmax(this_board, level - 1, 1 - is_red);
            this_score = result.score;
        }

        if (this_score > score)
        {
            best = this_move;
            score = this_score;
        }

        free(this_board);

        ptr = ptr->next;
    }

    free_node(moves);

    if (score == -10000000)
    {
        printf("NO!!!");
    }

    struct Result r;

    r.m = best;
    r.score = score;

    return r;
}