// A token that represents a single unit of Hinton code.
#[derive(Clone)]
pub struct Token {
   /// The token's line number
   pub line_num: usize,
   /// The token's column start
   pub column_start: usize,
   /// The token's column end
   pub column_end: usize,
   /// The token's type
   pub token_type: TokenType,
   /// The token's lexeme
   pub lexeme: String,
}

/// The types of tokens in a Hinton program.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
#[repr(u8)]
pub enum TokenType {
   AS_OPERATOR,
   BINARY,
   BIT_AND,
   BIT_AND_EQ,
   BIT_L_SHIFT,
   BIT_L_SHIFT_EQ,
   BIT_NOT,
   BIT_OR,
   BIT_OR_EQ,
   BIT_R_SHIFT,
   BIT_R_SHIFT_EQ,
   BIT_XOR,
   BIT_XOR_EQ,
   BREAK_KW,
   CLASS_KW,
   COLON,
   COLON_EQUALS,
   COMMA,
   CONST_KW,
   CONTINUE_KW,
   DOT,
   ELSE_KW,
   ENUM_KW,
   EOF,
   EQUALS,
   ERROR,
   EXPO,
   EXPO_EQUALS,
   FALSE,
   FLOAT,
   FN_LAMBDA_KW,
   FOR_KW,
   FUNC_KW,
   GREATER_THAN,
   GREATER_THAN_EQ,
   HEXADECIMAL,
   IDENTIFIER,
   IF_KW,
   INTEGER,
   IN_KW,
   LESS_THAN,
   LESS_THAN_EQ,
   LET_KW,
   LOGIC_AND,
   LOGIC_EQ,
   LOGIC_NOT,
   LOGIC_NOT_EQ,
   LOGIC_OR,
   L_BRACKET,
   L_CURLY,
   L_PAREN,
   MINUS,
   MINUS_EQ,
   MODULUS,
   MOD_EQ,
   NEW_KW,
   NULL,
   NULLISH,
   OCTAL,
   PLUS,
   PLUS_EQ,
   PRIVATE_KW,
   PUBLIC_KW,
   QUESTION,
   RANGE_OPR,
   RETURN_KW,
   R_BRACKET,
   R_CURLY,
   R_PARENTHESIS,
   SELF_KW,
   SEMICOLON,
   SLASH,
   SLASH_EQ,
   STAR,
   STAR_EQ,
   STRING,
   SUPER_KW,
   THIN_ARROW,
   TRUE,
   WHILE_KW,

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

impl TokenType {
   /// Checks that this token is of a given type.
   ///
   /// # Parameters
   /// - `token_type`: The token type to be matched against this token.
   pub fn type_match(&self, token_type: &TokenType) -> bool {
      if std::mem::discriminant(self) == std::mem::discriminant(token_type) {
         true
      } else {
         false
      }
   }
}

/// Maps a keyword string to a token type.
///
/// # Parameters
/// - `id`: The identifier's string name.
///
/// # Returns
/// `TokenType`: The type of token matched for given identifier name.
pub fn make_identifier_type(id: &str) -> TokenType {
   return match id {
      "and" => TokenType::LOGIC_AND,
      "as" => TokenType::AS_OPERATOR,
      "break" => TokenType::BREAK_KW,
      "class" => TokenType::CLASS_KW,
      "const" => TokenType::CONST_KW,
      "continue" => TokenType::CONTINUE_KW,
      "else" => TokenType::ELSE_KW,
      "enum" => TokenType::ENUM_KW,
      "equals" => TokenType::LOGIC_EQ,
      "false" => TokenType::FALSE,
      "fn" => TokenType::FN_LAMBDA_KW,
      "for" => TokenType::FOR_KW,
      "func" => TokenType::FUNC_KW,
      "if" => TokenType::IF_KW,
      "in" => TokenType::IN_KW,
      "let" => TokenType::LET_KW,
      "mod" => TokenType::MODULUS,
      "new" => TokenType::NEW_KW,
      "not" => TokenType::LOGIC_NOT,
      "null" => TokenType::NULL,
      "or" => TokenType::LOGIC_OR,
      "private" => TokenType::PRIVATE_KW,
      "public" => TokenType::PUBLIC_KW,
      "return" => TokenType::RETURN_KW,
      "self" => TokenType::SELF_KW,
      "super" => TokenType::SUPER_KW,
      "true" => TokenType::TRUE,
      "while" => TokenType::WHILE_KW,

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
