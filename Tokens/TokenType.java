package org.hinton_lang.Tokens;

public enum TokenType {
    // Operators
    PLUS, MINUS, MULT, DIV, EXPO, MOD, GREATER_THAN, GREATER_THAN_EQ, LESS_THAN, LESS_THAN_EQ, EQUALS_SIGN, LOGICAL_NOT,
    LOGICAL_AND, LOGICAL_OR, LOGICAL_EQ, LOGICAL_NOT_EQ, LOGICAL_IS, BITWISE_SHIFT_LEFT, BITWISE_SHIFT_RIGHT,
    BITWISE_AND, BITWISE_OR, BITWISE_NOT, BITWISE_XOR, RANGE_OPERATOR, IN_OPERATOR, AS_OPERATOR,

    // Keywords and Identifiers
    LET_KEYWORD, FUNC_KEYWORD, CONST_KEYWORD, IF_KEYWORD, ELSE_KEYWORD, IDENTIFIER, CLASS_KEYWORD, FOR_KEYWORD,
    WHILE_KEYWORD, RETURN_KEYWORD, LOOP_KEYWORD, BREAK_KEYWORD, CONTINUE_KEYWORD, INSTANCE_OF_KEYWORD, IMPORT_KEYWORD,
    FROM_KEYWORD, EXPORT_KEYWORD, NEW_KEYWORD, INIT_KEYWORD, FINAL_KEYWORD, PUBLIC_KEYWORD, PRIVATE_KEYWORD,
    STATIC_KEYWORD, ABSTRACT_KEYWORD, OPTIONAL_KEYWORD, OVERRIDE_KEYWORD, SELF_KEYWORD, IMPLEMENTS_KEYWORD,
    EXTENDS_KEYWORD, ASYNC_KEYWORD, AWAIT_KEYWORD, YIELD_KEYWORD, ENUM_KEYWORD, STRUCT_KEYWORD, INTERFACE_KEYWORD,
    FLEX_KEYWORD,

    // Types
    INTEGER_TYPE, REAL_TYPE, STRING_TYPE, CHARACTER_TYPE, BOOLEAN_TYPE, DICTIONARY_TYPE, SET_TYPE, FUNCTION_TYPE,
    VOID_TYPE, ANY_TYPE, NULL_TYPE,

    // Separators and Delimiters
    L_PARENTHESIS, R_PARENTHESIS, L_CURLY_BRACES, R_CURLY_BRACES, COLON_SEPARATOR, SEMICOLON_SEPARATOR, COMMA_SEPARATOR,
    L_SQUARE_BRACKET, R_SQUARE_BRACKET, DOT_SEPARATOR,

    // Value Literals
    NULL_LITERAL, BOOL_LITERAL_TRUE, BOOL_LITERAL_FALSE, STRING_LITERAL, INTEGER_LITERAL, REAL_LITERAL, CHAR_LITERAL,

    // Others
    BAD_CHARACTER, LINE_COMMENT, INLINE_COMMENT, BLOCK_COMMENT, WHITESPACE, END_OF_FILE
}
