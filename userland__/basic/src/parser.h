#ifndef PARSER_H
#define PARSER_H

#include <stdbool.h>

typedef enum TokenType
{
    NUMBER_LITERAL,
    STRING_LITERAL,
    SYMBOL,
    IDENTIFIER
} TokenType;

typedef struct Token
{
    TokenType type;
    union TokenData
    {
        char* text;
        int number;
    } data;
} Token;

Token* construct_token_string(TokenType type, char* text);
Token* construct_token_number(int number);
void drop_token(Token* token);

bool check_token(Token* token, TokenType type, const char* text);
Token* parse_next_token(char** text);
void display_token(Token* token);

#endif // PARSER_H