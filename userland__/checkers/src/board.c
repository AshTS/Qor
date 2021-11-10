#include "board.h"

#include "malloc.h"
#include "printf.h"

#define FORE_RED "\x1B[31m"
#define FORE_BLACK "\x1B[30m"
#define BACK_WHITE "\x1B[47m"
#define BACK_BLACK "\x1B[40m"
#define CLEAR "\x1B[0m"

Board* alloc_board()
{
    return malloc(sizeof(Board));
}

void free_board(Board* board)
{
    free(board);
}

Node* alloc_node(Move move, Node* next)
{
    Node* node = malloc(sizeof(Node));

    node->move = move;
    node->next = next;

    return node;
}

void free_node(Node* node)
{
    Node* next = node->next;

    free(node);

    if (next != 0)
    {
        free_node(next);
    }
}

Move move(int x0, int y0, int x1, int y1)
{
    Move m;

    m.start = (x0 + y0 * 8) / 2;
    m.end = (x1 + y1 * 8) / 2;

    return m;
}

void reset_board(Board* board)
{
    for (int i = 0; i < 32; i++)
    {
        if (i < 12)
        {
            board->cells[i] = TAKEN | RED;
        }
        else if (i > 19)
        {
            board->cells[i] = TAKEN | BLACK;
        }
        else
        {
            board->cells[i] = EMPTY;
        }
    }
}

void display_board(Board* board)
{
    printf("\n   A B C D E F G H");

    for (int i = 0; i < 64; i++)
    {
        int y = i / 8;
        int x = i % 8;

        if (i % 8 == 0)
        {
            printf("%s\n %i ", CLEAR, y + 1);
        }

        CellEntry piece = ((x + y) % 2 == 1) ? board->cells[i / 2] : 0;

        if (!(piece & TAKEN))
        {
            piece = 0;
        }

        printf("%s%s %s", 
            // Cell Color
            ((x + y) % 2 == 0) ? BACK_BLACK : BACK_WHITE,
            // Piece Color
            piece & RED ? FORE_RED : FORE_BLACK,
            // Piece
            piece & TAKEN ? ((piece & KING) ? "O" : "o") : " "
            );
    }

    printf("\n%s", CLEAR);
}

int make_move(Board* board, Move move)
{
    board->cells[move.end] = board->cells[move.start];
    board->cells[move.start] = 0;

    if (board->cells[move.end] & TAKEN)
    {
        if (((board->cells[move.end] & RED) && (move.end / 4) == 7) || 
            ((move.end / 4) == 0))
        {
            board->cells[move.end] |= KING;
        }
    }

    int val = move.end - move.start;

    int row_add = (move.start / 4) % 2 == 0 ? 0 : -1;
    int offset = 0;

    if (val == -9)
    {
        offset = -4 + row_add;
    }
    else if (val == -7)
    {
        offset = -3 + row_add;
    }
    else if (val == 7)
    {
        offset = 4 + row_add;
    }
    else if (val == 9)
    {
        offset = 5 + row_add;
    }

    if (offset != 0)
    {
        board->cells[move.start + offset] = 0;

        return 1;
    }

    return 0;
}

int evaluate_board(Board* board, char is_red)
{
    int eval = 0;

    for (int i = 0; i < 32; i++)
    {
        if (board->cells[i] & TAKEN)
        {
            int this = 10;

            if (board->cells[i] & KING)
            {
                this += 5;
            }

            int y = 7 - (i / 4);

            if (board->cells[i] & RED)
            {
                y = 7 - y;
            }

            this += y;

            if (board->cells[i] & RED)
            {
                this *= -1;
            }

            eval += this;
        }
    }

    return is_red ? -eval : eval;
}

char check_free(Board* board, char loc)
{
    return !(board->cells[loc] & TAKEN);
}

char check_other(Board* board, char loc, char is_red)
{
    if (check_free(board, loc))
    {
        return 0;
    }

    CellEntry cell = board->cells[loc];

    if (is_red)
    {
        return !(cell & RED);
    }
    else
    {
        return cell & RED;
    }
}

Node* get_moves(Board* board, char is_red)
{
    Node* moves = 0;

    for (int i = 0; i < 32; i++)
    {
        if (board->cells[i] & TAKEN)
        {
            if ((is_red && board->cells[i] & RED) || ((!is_red) && !(board->cells[i] & RED)))
            {
                int row_add = (i / 4) % 2 == 0 ? 0 : -1;
                int check_add = (i / 4) % 2 == 0 ? 1 : 0;
                int check_add_not = (i / 4) % 2 == 0 ? 0 : -1;

                int ul = i - 4 + row_add;
                int ur = i - 3 + row_add;
                int dl = i + 4 + row_add;
                int dr = i + 5 + row_add;

                char up = !is_red;
                char down = is_red;

                if (board->cells[i] & KING)
                {
                    up = 1;
                    down = 1;
                }

                if (up)
                {
                    if (i > 3 && ((i % 4) + check_add) > 0 && check_free(board, ul))
                    {
                        Move m;
                        m.start = i;
                        m.end = ul;

                        moves = alloc_node(m, moves);
                    }
                    else if (i > 7 && ((i % 4) + check_add) > 1 && check_other(board, ul, is_red) && check_free(board, i - 9))
                    {
                        Move m;
                        m.start = i;
                        m.end = i - 9;

                        moves = alloc_node(m, moves);
                    }

                    if (i > 3 && ((i % 4) + check_add_not) < 3 && check_free(board, ur))
                    {
                        Move m;
                        m.start = i;
                        m.end = ur;

                        moves = alloc_node(m, moves);
                    }
                    else if (i > 7 && ((i % 4) + check_add_not) < 2 && check_other(board, ur, is_red) && check_free(board, i - 7))
                    {
                        Move m;
                        m.start = i;
                        m.end = i - 7;

                        moves = alloc_node(m, moves);
                    }
                }

                if (down)
                {
                    if (i < 27 && ((i % 4) + check_add) > 0 && check_free(board, dl))
                    {
                        Move m;
                        m.start = i;
                        m.end = dl;

                        moves = alloc_node(m, moves);
                    }
                    else if (i < 24 &&((i % 4) + check_add) > 1 && check_other(board, dl, is_red) && check_free(board, i + 7))
                    {
                        Move m;
                        m.start = i;
                        m.end = i + 7;

                        moves = alloc_node(m, moves);
                    }

                    if (i < 27 && ((i % 4) + check_add_not) < 3 && check_free(board, dr))
                    {
                        Move m;
                        m.start = i;
                        m.end = dr;

                        moves = alloc_node(m, moves);
                    }
                    else if (i < 24 && ((i % 4) + check_add_not) < 2 && check_other(board, dr, is_red) && check_free(board, i + 9))
                    {
                        Move m;
                        m.start = i;
                        m.end = i + 9;

                        moves = alloc_node(m, moves);
                    }
                }
            }
        }
    }

    return moves;
}