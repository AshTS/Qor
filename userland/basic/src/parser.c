#include "parser.h"

#include <malloc.h>
#include <printf.h>
#include <string.h>

bool is_alpha(char);
bool is_num(char);

bool is_alpha_num(char c)
{
    return is_alpha(c) | is_num(c);
}

int parse_number(char* text, bool* could_parse, int* num_chars)
{
    bool is_neg = false;
    int result = 0;
    *num_chars = 0;

    if (*text == '-')
    {
        text++;
        (*num_chars)++;
        is_neg = true;
    }

    if (!is_num(*text))
    {
        *could_parse = false;
        return 0;
    }

    while (is_num(*text))
    {
        result *= 10;
        result += (*text - '0');
        text++;
        (*num_chars)++;
    }

    *could_parse = true;
    return result * (is_neg ? -1 : 1);
}

Token* construct_token_string(TokenType type, char* text)
{
    Token* result = malloc(sizeof(Token));
    char* buffer = malloc(strlen(text) + 1);
    strcpy(buffer, text);
    result->type = type;
    result->data.text = buffer;

    return result;
}

Token* construct_token_number(int number)
{
    Token* result = malloc(sizeof(Token));
    result->type = NUMBER_LITERAL;
    result->data.number = number;

    return result;
}

bool check_token(Token* token, TokenType type, const char* text)
{
    if (token->type != type)
    {
        return false;
    }

    return strcmp(text, token->data.text) == 0;
}

void drop_token(Token* token)
{
    if (token->type != NUMBER_LITERAL)
    {
        free(token->data.text);
    }
    free(token);
}

Token* parse_next_token(char** text)
{
    // Step past any white space
    while (**text == ' ' || **text == '\t')
    {
        (*text)++;
    }

    // Return null if no more tokens are in the string
    if (**text == 0)
    {
        return 0;
    }

    // First attempt to parse a number
    bool success;
    int num_chars;
    int symbol = parse_number(*text, &success, &num_chars);

    // If a number was parsed
    if (success)
    {
        (*text) += num_chars;
        return construct_token_number(symbol);
    }

    // Buffer for identifiers or variable names
    char buffer[64];

    // Next check for any symbols
    switch (**text)
    {
        case '(':
        case ')':
        case ',':
        case '+':
        case '-':
        case '*':
        case '/':
            buffer[0] = **text;
            buffer[1] = 0;
            (*text)++;
            return construct_token_string(SYMBOL, buffer);
        case '<':
        case '>':
        case '=':
        case '!':
            buffer[0] = **text;
            if (*(*text + 1) == '=')
            {
                (*text)++;
                buffer[1] = **text;
                buffer[2] = 0;
            }
            else
            {
                buffer[1] = 0;
            }
            (*text)++;
            return construct_token_string(SYMBOL, buffer);
    }

    
    // Construct a string literal
    if (**text == '"')
    {
        int index = 0;
        (*text)++;

        while (**text != '"')
        {
            if (**text == 0)
            {
                eprintf("Unexpected end of line while parsing!\n");
                return 0;
            }

            char v;
            
            if (**text == '\\')
            {
                (*text)++;

                switch (**text)
                {
                    case '"':
                        v = '"';
                        break;
                    case '\\':
                        v = '\\';
                        break;
                    case 'n':
                        v = '\n';
                        break;
                    case 't':
                        v = '\t';
                        break;
                    default:
                        eprintf("Unknown Escape Sequence `\\%c`\n", **text);
                        return 0;
                }

                (*text)++;
            }
            else
            {
                v = *((*text)++);
            }

            buffer[index++] = v;
        }

        (*text)++;

        buffer[index] = 0;

        return construct_token_string(STRING_LITERAL, buffer);
    }

    // Construct an identifier
    if (is_alpha(**text) || **text == '_')
    {
        int index = 0;

        while (is_alpha_num(**text) || **text == '_')
        {
            buffer[index++] = *((*text)++);
        }

        buffer[index] = 0;

        return construct_token_string(IDENTIFIER, buffer);
    }
    else
    {
        eprintf("Cannot parse token from `%s`\n", *text);
    }

    return 0;
}

void display_token(Token* token)
{

    switch (token->type)
    {
        case NUMBER_LITERAL:
            printf("NUMBER_LITERAL(`%i`)\n", token->data.number);
            break;
        case STRING_LITERAL:
            printf("STRING_LITERAL(`%s`)\n", token->data.text);
            break;
        case SYMBOL:
            printf("SYMBOL(`%s`)\n", token->data.text);
            break;
        case IDENTIFIER:
            printf("IDENTIFIER(`%s`)\n", token->data.text);
            break;
        default:
            printf("UNKNOWN\n");
    }
}

bool is_alpha(char c)
{
    return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z');
}


bool is_num(char c)
{
    return c >= '0' && c <= '9';
}