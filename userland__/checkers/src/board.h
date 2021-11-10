#ifndef BOARD_H
#define BOARD_H

// Board Cell Entry
enum CellEntry
{
    EMPTY = 0,
    TAKEN = 1,
    BLACK = 0,
    RED = 2,
    KING = 4
};

typedef enum CellEntry CellEntry;

// Board Object
struct Board
{
    CellEntry cells[32];
};

typedef struct Board Board;

// Move Object
struct Move
{
    int start;
    int end;
};

typedef struct Move Move;

// Move Node List
struct Node
{
    Move move;
    struct Node* next;
};

typedef struct Node Node;

Board* alloc_board();
void free_board(Board* board);

Node* alloc_node(Move move, Node* next);
void free_node(Node* node);

Move move(int x0, int y0, int x1, int y1);

void reset_board(Board* board);
void display_board(Board* board);
int make_move(Board* board, Move move);

int evaluate_board(Board* board, char is_red);

Node* get_moves(Board* board, char is_red);

#endif // BOARD_H