use crate::lexer::tokens::Token;
use crate::lexer::tokens::TokenType::*;
use crate::lexer::Lexer;

impl Lexer {
   /// Scans the next token in the source file.
   pub fn next_token(&mut self) -> Token {
      self.skip_whitespace();

      // If we are at the end, return the EOF token.
      if self.is_at_end() {
         return self.make_token(EOF);
      }

      // Reset the start of the token
      self.token_start = self.current;
      let c = self.advance();

      // Generates an identifier/keyword if the current character is alphanumeric
      if c.is_alphabetic() {
         return self.make_identifier_token();
      }

      // Generates a numeric literal if the current character is alphanumeric
      if c.is_digit(10) {
         return self.make_numeric_token();
      }

      // Generate symbol-like token tokens
      return match c {
         '"' | '\'' => self.make_string_token(),
         '(' => self.make_token(L_PAREN),
         ')' => self.make_token(R_PARENTHESIS),
         '{' => self.make_token(L_CURLY),
         '}' => self.make_token(R_CURLY),
         '[' => self.make_token(L_BRACKET),
         ']' => self.make_token(R_BRACKET),
         ';' => self.make_token(SEMICOLON),
         ',' => self.make_token(COMMA),
         '~' => self.make_token(BIT_NOT),
         '/' => {
            let tok = if self.matches('=') { SLASH_EQ } else { SLASH };
            self.make_token(tok)
         }
         '%' => {
            let tok = if self.matches('=') { MOD_EQ } else { MODULUS };
            self.make_token(tok)
         }
         '!' => {
            let tok = if self.matches('=') {
               LOGIC_NOT_EQ
            } else {
               LOGIC_NOT
            };
            self.make_token(tok)
         }
         '=' => {
            let tok = if self.matches('=') { LOGIC_EQ } else { EQUALS };
            self.make_token(tok)
         }
         ':' => {
            let tok = if self.matches('=') { COLON_EQUALS } else { COLON };
            self.make_token(tok)
         }
         '^' => {
            let tok = if self.matches('=') { BIT_XOR_EQ } else { BIT_XOR };
            self.make_token(tok)
         }
         '&' => {
            let tok = if self.matches('&') {
               LOGIC_AND
            } else if self.matches('=') {
               BIT_AND_EQ
            } else {
               BIT_AND
            };
            self.make_token(tok)
         }
         '|' => {
            let tok = if self.matches('|') {
               LOGIC_OR
            } else if self.matches('=') {
               BIT_OR_EQ
            } else {
               BIT_OR
            };
            self.make_token(tok)
         }
         '?' => {
            if self.matches('?') {
               self.make_token(NULLISH)
            }
            // else if self.matches(':') {
            //     self.make_token(ELVIS_OPERATOR)
            // }
            else {
               self.make_token(QUESTION)
            }
         }
         '.' => {
            if self.get_current().is_digit(10) {
               self.make_numeric_token()
            } else if self.matches('.') {
               self.make_token(RANGE_OPR)
            } else {
               self.make_token(DOT)
            }
         }
         '-' => {
            if self.matches('=') {
               self.make_token(MINUS_EQ)
            } else if self.matches('>') {
               self.make_token(THIN_ARROW)
            } else {
               self.make_token(MINUS)
            }
         }
         '+' => {
            if self.matches('=') {
               self.make_token(PLUS_EQ)
            } else {
               self.make_token(PLUS)
            }
         }
         '*' => {
            if self.matches('=') {
               self.make_token(STAR_EQ)
            } else if self.matches('*') {
               if self.matches('=') {
                  self.make_token(EXPO_EQUALS)
               } else {
                  self.make_token(EXPO)
               }
            } else {
               self.make_token(STAR)
            }
         }
         '<' => {
            if self.matches('=') {
               self.make_token(LESS_THAN_EQ)
            } else if self.matches('<') {
               if self.matches('=') {
                  self.make_token(BIT_L_SHIFT_EQ)
               } else {
                  self.make_token(BIT_L_SHIFT)
               }
            } else {
               self.make_token(LESS_THAN)
            }
         }
         '>' => {
            if self.matches('=') {
               self.make_token(GREATER_THAN_EQ)
            } else if self.matches('>') {
               if self.matches('=') {
                  self.make_token(BIT_R_SHIFT_EQ)
               } else {
                  self.make_token(BIT_R_SHIFT)
               }
            } else {
               self.make_token(GREATER_THAN)
            }
         }

         // Everything else is an error token
         _ => self.make_error_token("Unexpected character"),
      };
   }
}
