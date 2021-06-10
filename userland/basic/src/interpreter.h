#ifndef INTERPRETER_H
#define INTERPRETER_H

#include <stdbool.h>

#include "parser.h"

#define START_LINE_COUNT 512

typedef union Data
{
    char* string;
    int number;
} Data;

typedef enum DataType
{
    STRING,
    NUMBER
} DataType;

typedef struct Value
{
    DataType type;
    Data data;
    bool valid;
} Value;

typedef struct VariableEntry
{
    char* name;
    Value value;
    struct VariableEntry* next;
} VariableEntry;

typedef struct InterpreterState
{
    unsigned int line_count;
    Token*** lines;
    unsigned int current_line;
    VariableEntry* variables;
} InterpreterState;

InterpreterState* construct_interpreter();
void drop_interpreter(InterpreterState*);

bool interpret(InterpreterState*, Token** line);

void clear_interpreter(InterpreterState* state);
void run(InterpreterState* state);
void list(InterpreterState* state);


#endif // INTERPRETER_H