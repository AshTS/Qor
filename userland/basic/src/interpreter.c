#include "interpreter.h"

#include <malloc.h>
#include <printf.h>
#include <string.h>
#include <syscalls.h>

InterpreterState* construct_interpreter()
{
    InterpreterState* state = malloc(sizeof(InterpreterState));

    state->line_count = START_LINE_COUNT;
    state->lines = malloc(sizeof(Token**) * START_LINE_COUNT);

    for (int i = 0; i < state->line_count; i++)
    {
        state->lines[i] = 0;
    }

    state->variables = 0;

    return state;
}

void drop_line(Token** line)
{
    Token** orig_line = line;
    if (line != 0)
    {
        // Free each token
        while (*line != 0)
        {
            drop_token(*line);
            line++;
        }

        // Free the line
        free(orig_line);
    }
}

VariableEntry* construct_variable(char* name, Value value, VariableEntry* next)
{
    VariableEntry* entry = malloc(sizeof(VariableEntry));
    char* buffer = malloc(strlen(name) + 1);
    strcpy(buffer, name);

    entry->name = buffer;

    if (value.type == NUMBER)
    {
        entry->value = value;
    }
    else
    {
        char* value_buffer = malloc(strlen(value.data.string) + 1);
        strcpy(value_buffer, value.data.string);

        entry->value.type = value.type;
        entry->value.valid = value.valid;
        entry->value.data.string = value_buffer;
    }

    return entry;
}

void drop_variable(VariableEntry* entry)
{
    free(entry->name);

    if (entry->value.type == STRING)
    {
        free(entry->value.data.string);
    }

    free(entry);
}

void clear_interpreter(InterpreterState* state)
{
    // Free individual lines
    for (int i = 0; i < state->line_count; i++)
    {   
        // Free the tokens
        Token** line = state->lines[i];

        // Drop the tokens from the line and the line itself
        drop_line(line);
    }

    // Iterate over the variables, and drop them
    VariableEntry* var = state->variables;
    while (var)
    {
        VariableEntry* next = var->next;

        drop_variable(var);
        var = next;
    }
}

void drop_interpreter(InterpreterState* state)
{
    // Free individual lines
    for (int i = 0; i < state->line_count; i++)
    {   
        // Free the tokens
        Token** line = state->lines[i];

        // Drop the tokens from the line and the line itself
        drop_line(line);
    }

    // Free the lines array
    free(state->lines);

    // Iterate over the variables, and drop them
    VariableEntry* var = state->variables;
    while (var)
    {
        VariableEntry* next = var->next;

        drop_variable(var);
        var = next;
    }

    // Free the state
    free(state);
}

void list(InterpreterState* state)
{
    printf("List:\n");
    for (int i = 0; i < state->line_count; i++)
    {
        Token** line = state->lines[i];

        if (line != 0)
        {
            printf("%i\t", i);

            while (*line != 0)
            {
                Token* token = *line;

                if (token->type == NUMBER_LITERAL)
                {
                    printf("%i ", token->data.number);
                }
                else if (token->type == STRING_LITERAL)
                {
                    printf("\"%s\" ", token->data.text);
                }
                else
                {
                    printf("%s ", token->data.text);
                }

                line++;
            }

            printf("\n");
        }
    }
}

void run(InterpreterState* state)
{
    for (state->current_line = 0; state->current_line < state->line_count;)
    {
        Token** line = state->lines[state->current_line];

        if (line != 0)
        {
            if (!interpret(state, line))
            {
                break;
            }
        }
        else
        {
            state->current_line++;
        }
    }
}

void display_value(Value* v)
{
    if (v == 0)
    {
        printf("NULL");
        return;
    }

    if (!v->valid)
    {
        printf("INVALID");
        return;
    }

    if (v->type == NUMBER)
    {
        printf("NUM %i", v->data.number);
        return;
    }
    else
    {
        printf("STRING %s", v->data.string);
        return;
    }
}

void dump_variables(InterpreterState* state)
{
    VariableEntry* var = state->variables;

    if (var == 0)
    {
        printf("EMPTY\n");
    }

    while (var)
    {
        printf("`%s` => ", var->name);
        display_value(&var->value);
        printf("\n");

        var = var->next;
    }
}

Value get_variable(InterpreterState* state, char* name)
{
    VariableEntry* var = state->variables;

    while (var)
    {
        if (strcmp(var->name, name) == 0)
        {
            return var->value;
        }
        var = var->next;
    }

    Value v;
    printf("Could not find variable named `%s`\n", name);
    v.valid = false;

    return v;
}

void set_variable(InterpreterState* state, char* name, Value v)
{
    VariableEntry* this = construct_variable(name, v, 0);

    VariableEntry* var = state->variables;
    VariableEntry** last_next = &state->variables;

    while (var)
    {
        if (strcmp(var->name, name) == 0)
        {
            *last_next = this;
            this->next = var->next;

            drop_variable(var);

            return;
        }

        last_next = &var->next;
        var = var->next;
    }

    *last_next = this;
}

bool is_at_eof(Token** line, unsigned int* index, bool display_error)
{
    if (line[*index] == 0)
    {
        if (display_error)
        {
            printf("Unexpected EOF while parsing\n");
        }

        return true;
    }

    return false;
}

// Expression Levels
// 0 : String Literals, Number Literals, Parens
// 1 : Multiplication, Division
// 2 : Addition, Subtraction
// 3 : Lesser, Greater, Less Than, Greater Than, Equal, Not Equal

static const char MAX_LEVEL = 3;

Value evaluate_expression(InterpreterState* state, Token** line, unsigned int* index, char level)
{
    Value v;
    v.valid = true;

    if (is_at_eof(line, index, true))
    {
        v.valid = false;
        return v;
    }

    if (level == 0)
    {
        if (line[*index]->type == NUMBER_LITERAL)
        {
            v.type = NUMBER;
            v.data.number = line[*index]->data.number;
            (*index)++;

            return v;
        }
        else if (line[*index]->type == STRING_LITERAL)
        {
            v.type = STRING;
            v.data.string = line[*index]->data.text;
            (*index)++;

            return v;
        }
        else if (line[*index]->type == IDENTIFIER)
        {
            v = get_variable(state, line[*index]->data.text);
            (*index)++;

            return v;
        }
        else if (check_token(line[*index], SYMBOL, "("))
        {
            (*index)++;

            v = evaluate_expression(state, line, index, MAX_LEVEL);

            if (!v.valid)
            {
                return v;
            }

            if (is_at_eof(line, index, true))
            {
                v.valid = false;
                return v;
            }

            if (!check_token(line[*index], SYMBOL, ")"))
            {
                printf("Expected ')'\n");
                v.valid = false;

                return v;
            }

            (*index)++;

            return v;
        }
        else
        {
            printf("Unable to parse expression starting with token ");
            display_token(line[*index]);
            v.valid = false;

            return v;
        }
    }
    else if (level == 1)
    {
        Value child0 = evaluate_expression(state, line, index, level - 1);

        if (!child0.valid)
        {
            return child0;
        }

        if (is_at_eof(line, index, false))
        {
            return child0;
        }

        if (check_token(line[*index], SYMBOL, "*"))
        {
            (*index)++;

            Value child1 = evaluate_expression(state, line, index, level);

            if (!child1.valid)
            {
                return child1;
            }

            if (child0.type == STRING || child1.type == STRING)
            {
                printf("Unable to multiply with strings\n");
                v.valid = false;
                return v;
            }

            v.type = NUMBER;
            v.data.number = child0.data.number * child1.data.number;

            return v;
        }
        else if (check_token(line[*index], SYMBOL, "/"))
        {
            (*index)++;

            Value child1 = evaluate_expression(state, line, index, level);

            if (!child1.valid)
            {
                return child1;
            }

            if (child0.type == STRING || child1.type == STRING)
            {
                printf("Unable to divide with strings\n");
                v.valid = false;
                return v;
            }

            v.type = NUMBER;
            v.data.number = child0.data.number / child1.data.number;

            return v;
        }

        return child0;
    }
    else if (level == 2)
    {
        Value child0 = evaluate_expression(state, line, index, level - 1);

        if (!child0.valid)
        {
            return child0;
        }

        if (is_at_eof(line, index, false))
        {
            return child0;
        }

        if (check_token(line[*index], SYMBOL, "+"))
        {
            (*index)++;

            Value child1 = evaluate_expression(state, line, index, level);

            if (!child1.valid)
            {
                return child1;
            }

            if (child0.type == STRING || child1.type == STRING)
            {
                printf("Unable to add with strings\n");
                v.valid = false;
                return v;
            }

            v.type = NUMBER;
            v.data.number = child0.data.number + child1.data.number;

            return v;
        }
        else if (check_token(line[*index], SYMBOL, "-"))
        {
            (*index)++;

            Value child1 = evaluate_expression(state, line, index, level);

            if (!child1.valid)
            {
                return child1;
            }

            if (child0.type == STRING || child1.type == STRING)
            {
                printf("Unable to subtract with strings\n");
                v.valid = false;
                return v;
            }

            v.type = NUMBER;
            v.data.number = child0.data.number - child1.data.number;

            return v;
        }

        return child0;
    }
    else if (level == 3)
    {
        Value child0 = evaluate_expression(state, line, index, level - 1);

        if (!child0.valid)
        {
            return child0;
        }

        if (is_at_eof(line, index, false))
        {
            return child0;
        }

        if (check_token(line[*index], SYMBOL, "<"))
        {
            (*index)++;

            Value child1 = evaluate_expression(state, line, index, level);

            if (!child1.valid)
            {
                return child1;
            }

            if (child0.type == STRING || child1.type == STRING)
            {
                printf("Unable to compare strings outside of equality\n");
                v.valid = false;
                return v;
            }

            v.type = NUMBER;
            v.data.number = child0.data.number < child1.data.number;

            return v;
        }
        else if (check_token(line[*index], SYMBOL, ">"))
        {
            (*index)++;

            Value child1 = evaluate_expression(state, line, index, level);

            if (!child1.valid)
            {
                return child1;
            }

            if (child0.type == STRING || child1.type == STRING)
            {
                printf("Unable to compare strings outside of equality\n");
                v.valid = false;
                return v;
            }

            v.type = NUMBER;
            v.data.number = child0.data.number > child1.data.number;

            return v;
        }
        else if (check_token(line[*index], SYMBOL, "<="))
        {
            (*index)++;

            Value child1 = evaluate_expression(state, line, index, level);

            if (!child1.valid)
            {
                return child1;
            }

            if (child0.type == STRING || child1.type == STRING)
            {
                printf("Unable to compare strings outside of equality\n");
                v.valid = false;
                return v;
            }

            v.type = NUMBER;
            v.data.number = child0.data.number <= child1.data.number;

            return v;
        }
        else if (check_token(line[*index], SYMBOL, ">="))
        {
            (*index)++;

            Value child1 = evaluate_expression(state, line, index, level);

            if (!child1.valid)
            {
                return child1;
            }

            if (child0.type == STRING || child1.type == STRING)
            {
                printf("Unable to compare strings outside of equality\n");
                v.valid = false;
                return v;
            }

            v.type = NUMBER;
            v.data.number = child0.data.number >= child1.data.number;

            return v;
        }
        else if (check_token(line[*index], SYMBOL, "=="))
        {
            (*index)++;

            Value child1 = evaluate_expression(state, line, index, level);

            if (!child1.valid)
            {
                return child1;
            }

            if (child0.type == STRING && child1.type == STRING)
            {
                v.type = NUMBER;
                v.data.number = strcmp(child0.data.string, child1.data.string) == 0;

                return v;
            }
            else if (child0.type == NUMBER && child1.type == NUMBER)
            {
                v.type = NUMBER;
                v.data.number = child0.data.number == child1.data.number;

                return v;
            }
            else
            {
                printf("Unable to compare differing types\n");
                v.valid = false;
                return v;
            }
        }
        else if (check_token(line[*index], SYMBOL, "!="))
        {
            (*index)++;

            Value child1 = evaluate_expression(state, line, index, level);

            if (!child1.valid)
            {
                return child1;
            }

            if (child0.type == STRING && child1.type == STRING)
            {
                v.type = NUMBER;
                v.data.number = strcmp(child0.data.string, child1.data.string) != 0;

                return v;
            }
            else if (child0.type == NUMBER && child1.type == NUMBER)
            {
                v.type = NUMBER;
                v.data.number = child0.data.number != child1.data.number;

                return v;
            }
            else
            {
                printf("Unable to compare differing types\n");
                v.valid = false;
                return v;
            }
        }

        return child0;
    }

    v.valid = false;
    return v;
}

bool interpret(InterpreterState* state, Token** line)
{
    state->current_line++;

    if (line == 0)
    {
        return false;
    }

    // Check for commands (like list)
    if (check_token(*line, IDENTIFIER, "list"))
    {
        list(state);
        return true;
    }
    else if (check_token(*line, IDENTIFIER, "run"))
    {
        run(state);
        return true;
    }
    // Add a line if the current input starts with a number
    else if ((*line)->type == NUMBER_LITERAL)
    {
        unsigned int line_number =  (*line)->data.number;

        int number_of_tokens = 0;
        Token** walking = line + 1;

        while (*walking != 0)
        {
            walking++;
            number_of_tokens++;
        }

        Token** token_space;
        
        if (number_of_tokens > 0)
        {
            token_space = malloc(sizeof(Token*) * (number_of_tokens + 1));
            token_space[number_of_tokens] = 0;
            
            walking = line + 1;
            Token** walking_dest = token_space;

            while (*walking != 0)
            {
                if ((*walking)->type == NUMBER_LITERAL)
                {
                    *walking_dest = construct_token_number((*walking)->data.number);
                }
                else
                {
                    *walking_dest = construct_token_string((*walking)->type, (*walking)->data.text);
                }

                walking++;
                walking_dest++;
            }
        }
        else
        {
            token_space = 0;
        }
        

        if (line_number < state->line_count)
        {
            // drop_line(state->lines[line_number]);
            state->lines[line_number] = token_space;
        }
        else
        {
            printf("Line #%i is too big!", line_number);
            return false;
        }

        return true;
    }
    // Print Statements
    else if (check_token(*line, IDENTIFIER, "print"))
    {
        line += 1;

        unsigned int index = 0;
        while (line[index] != 0)
        {
            Value v = evaluate_expression(state, line, &index, MAX_LEVEL);

            if (!v.valid)
            {
                return false;
            }

            if (v.type == STRING)
            {
                printf("%s", v.data.string);
            }
            else
            {
                printf("%i", v.data.number);
            }
        }

        printf("\n");

        return true;
    }
    // Goto Statement
    else if (check_token(*line, IDENTIFIER, "goto"))
    {
        if (line[1] == 0)
        {
            printf("Goto requires an argument\n");
            return false;
        }

        if (line[1]->type != NUMBER_LITERAL)
        {
            printf("Goto requires an argument\n");
            return false;
        }

        state->current_line = line[1]->data.number;

        if (state->current_line >= state->line_count)
        {
            printf("Jumping out of bounds\n");
            return false;
        }

        return true;
    }
    // Assignment
    else if (line[0]->type == IDENTIFIER && line[1] != 0 && check_token(line[1], SYMBOL, "="))
    {
        unsigned int index = 2;
        Value v = evaluate_expression(state, line, &index, MAX_LEVEL);

        if (!v.valid)
        {
            return false;
        }

        if (!is_at_eof(line, &index, false))
        {
            printf("Excess tokens after assignment\n");
            return false;
        }

        set_variable(state, line[0]->data.text, v);

        return true;
    }
    // If Statements
    else if (check_token(*line, IDENTIFIER, "if"))
    {
        unsigned int index = 1;
        Value v = evaluate_expression(state, line, &index, MAX_LEVEL);

        if (!v.valid)
        {
            return false;
        }

        if (is_at_eof(line, &index, true))
        {
            return false;
        }

        if (!check_token(line[index], IDENTIFIER, "then"))
        {
            printf("Expected `then`\n");
            return false;
        }

        index += 1;

        if (is_at_eof(line, &index, true))
        {
            return false;
        }

        if (v.type == NUMBER)
        {
            if (v.data.number != 0)
            {
                return interpret(state, &line[index]);
            }
        }
        else
        {
            printf("Value given to if statement must be a number\n");
            return false;
        }

        return true;
    }
    return false;
}