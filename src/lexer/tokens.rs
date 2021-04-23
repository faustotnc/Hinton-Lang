// A token that represents a single unit of Hinton code.
pub struct Token {
    /// The token's line number
    pub line_num: usize,
    /// The token's column number
    pub column_num: usize,
    /// The token's type
    pub token_type: TokenType,
    /// The token's lexeme
    pub lexeme: String,
}

/// Implementation of Token methods
impl Token {
    /// Print's a console-friendly version of this token
    /// ## Arguments
    /// * `details` – Whether or not to print other information about the token.
    pub fn print(&self, details: bool) {
        print!("Token: \x1b[36m{:?}\x1b[0m", self.token_type);

        if details {
            println!(" \"{}\" at [{}:{}].", self.lexeme, self.line_num, self.column_num);
        } else {
            println!();
        }
    }
}

/// Represents the type of a token.
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    AS_OPERATOR,
    BINARY_LITERAL,
    BITWISE_AND,
    BITWISE_LEFT_SHIFT,
    BITWISE_NOT,
    BITWISE_OR,
    BITWISE_RIGHT_SHIFT,
    BITWISE_XOR,
    BREAK_KEYWORD,
    CLASS_KEYWORD,
    COLON_EQUALS,
    COLON_SEPARATOR,
    COMMA_SEPARATOR,
    CONST_KEYWORD,
    CONTINUE_KEYWORD,
    DECREMENT,
    DOT_SEPARATOR,
    ELSE_KEYWORD,
    ELVIS_OPERATOR,
    ENUM_KEYWORD,
    EOF,
    EQUALS_SIGN,
    ERROR,
    EXPO,
    EXPO_EQUALS,
    FALSE_LITERAL,
    FN_LAMBDA_KEYWORD,
    FOR_KEYWORD,
    FUNC_KEYWORD,
    GREATER_THAN,
    GREATER_THAN_EQ,
    HEXADECIMAL_LITERAL,
    IDENTIFIER,
    IF_KEYWORD,
    INCREMENT,
    IN_OPERATOR,
    LEFT_CURLY_BRACES,
    LEFT_PARENTHESIS,
    LEFT_SQUARE_BRACKET,
    LESS_THAN,
    LESS_THAN_EQ,
    LET_KEYWORD,
    LOGICAL_AND,
    LOGICAL_EQ,
    LOGICAL_NOT,
    LOGICAL_NOT_EQ,
    LOGICAL_OR,
    MINUS,
    MINUS_EQUALS,
    MODULUS,
    MOD_EQUALS,
    NEW_KEYWORD,
    NULLISH_COALESCING,
    NULL_LITERAL,
    NUMERIC_LITERAL,
    OCTAL_LITERAL,
    PLUS,
    PLUS_EQUALS,
    PRIVATE_KEYWORD,
    PUBLIC_KEYWORD,
    QUESTION_MARK,
    RANGE_OPERATOR,
    RETURN_KEYWORD,
    RIGHT_CURLY_BRACES,
    RIGHT_PARENTHESIS,
    RIGHT_SQUARE_BRACKET,
    SELF_KEYWORD,
    SEMICOLON_SEPARATOR,
    SLASH,
    SLASH_EQUALS,
    STAR,
    STAR_EQUALS,
    STRING_LITERAL,
    SUPER_KEYWORD,
    THIN_ARROW,
    TRUE_LITERAL,
    WHILE_KEYWORD,

    // TEMPORARY
    PRINT,
    // ***** To be implemented/considered
    // ABSTRACT_KEYWORD,
    // ASYNC_KEYWORD,
    // AWAIT_KEYWORD,
    // EXPORT_KEYWORD,
    // EXTENDS_KEYWORD,
    // FLEX_KEYWORD,
    // FROM_KEYWORD,
    // IMPLEMENTS_KEYWORD,
    // INSTANCE_OF_KEYWORD,
    // INTERFACE_KEYWORD,
    // IN_OPERATOR,
    // LOGICAL_IS,
    // LOOP_KEYWORD,
    // OPTIONAL_KEYWORD,
    // OVERRIDE_KEYWORD,
    // STATIC_KEYWORD,
    // ANY_TYPE,
    // BAD_CHARACTER,
    // BOOLEAN_TYPE,
    // CHAR_LITERAL,
    // DICTIONARY_TYPE,
    // FLOAT_TYPE,
    // FUNCTION_TYPE,
    // IMPORT_KEYWORD,
    // INTEGER_TYPE,
    // NULL_TYPE,
    // STRING_TYPE,
    // STRUCT_KEYWORD,
    // VOID_TYPE,
    // YIELD_KEYWORD

    // This one is only used to initialize the compiler
    __INIT_PARSER__,
}

/// Maps a keyword string to a token type.
/// Used for lexing Hinton keywords.
pub fn make_identifier_type(id: &str) -> TokenType {
    return match id {
        "and" => TokenType::LOGICAL_AND,
        "as" => TokenType::AS_OPERATOR,
        "break" => TokenType::BREAK_KEYWORD,
        "class" => TokenType::CLASS_KEYWORD,
        "const" => TokenType::CONST_KEYWORD,
        "continue" => TokenType::CONTINUE_KEYWORD,
        "else" => TokenType::ELSE_KEYWORD,
        "enum" => TokenType::ENUM_KEYWORD,
        "equals" => TokenType::LOGICAL_EQ,
        "false" => TokenType::FALSE_LITERAL,
        "fn" => TokenType::FN_LAMBDA_KEYWORD,
        "for" => TokenType::FOR_KEYWORD,
        "func" => TokenType::FUNC_KEYWORD,
        "if" => TokenType::IF_KEYWORD,
        "in" => TokenType::IN_OPERATOR,
        "let" => TokenType::LET_KEYWORD,
        "mod" => TokenType::MODULUS,
        "new" => TokenType::NEW_KEYWORD,
        "not" => TokenType::LOGICAL_NOT,
        "null" => TokenType::NULL_LITERAL,
        "or" => TokenType::LOGICAL_OR,
        "print" => TokenType::PRINT,
        "private" => TokenType::PRIVATE_KEYWORD,
        "public" => TokenType::PUBLIC_KEYWORD,
        "return" => TokenType::RETURN_KEYWORD,
        "self" => TokenType::SELF_KEYWORD,
        "super" => TokenType::SUPER_KEYWORD,
        "true" => TokenType::TRUE_LITERAL,
        "while" => TokenType::WHILE_KEYWORD,

        // ***** To be implemented/considered
        // "Any"       => TokenType::ANY_TYPE,
        // "Array"      => TokenType::ARRAY_DATATYPE,
        // "Bool"      => TokenType::BOOLEAN_TYPE,
        // "Char"       => TokenType::CHARACTER_TYPE,
        // "Dict"      => TokenType::DICTIONARY_TYPE,
        // "Float"     => TokenType::FLOAT_TYPE,
        // "Function"  => TokenType::FUNCTION_TYPE,
        // "Int"       => TokenType::INTEGER_TYPE,
        // "Null"      => TokenType::NULL_TYPE,
        // "String"    => TokenType::STRING_TYPE,
        // "Void"      => TokenType::VOID_TYPE,
        // "abstract"  => TokenType::ABSTRACT_KEYWORD,
        // "async"  => TokenType::ASYNC_KEYWORD,
        // "await"  => TokenType::AWAIT_KEYWORD,
        // "export"    => TokenType::EXPORT_KEYWORD,
        // "extends"   => TokenType::EXTENDS_KEYWORD,
        // "final"     => TokenType::FINAL_KEYWORD,
        // "from"      => TokenType::FROM_KEYWORD,
        // "implements"    => TokenType::IMPLEMENTS_KEYWORD,
        // "import"     => TokenType::IMPORT_KEYWORD,
        // "instanceOf"    => TokenType::INSTANCE_OF_KEYWORD,
        // "interface"  => TokenType::INTERFACE_KEYWORD,
        // "is"     => TokenType::IS_OPERATOR,
        // "optional"  => TokenType::OPTIONAL_KEYWORD,
        // "override"  => TokenType::OVERRIDE_KEYWORD,
        // "static"    => TokenType::STATIC_KEYWORD,
        // "struct"     => TokenType::STRUCT_KEYWORD,
        // "yield"      => TokenType::YIELD_KEYWORD,
        _ => TokenType::IDENTIFIER,
    };
}