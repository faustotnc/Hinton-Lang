use crate::lexer::tokens::{Token, TokenType};

// Submodules
pub mod lex_next_token;
pub mod lex_numbers;
pub mod tokens;

/// Struct that represents the scanner.
pub struct Lexer {
   /// A flat list of characters from the source file.
   source: Vec<char>,
   /// The index of the current character.
   current: usize,
   /// The current line index.
   line_num: usize,
   /// The position in the flat source vector of the first
   /// character for the current line.
   line_start: usize,
   /// The position in the flat source vector of the first
   /// character for the current token.
   token_start: usize,
}

impl Lexer {
   /// An initialized instance of the lexer.
   /// # Parameters
   /// - `src` - the source file
   ///
   /// ## Example
   /// ```
   /// let mut l = Lexer::lex("let x = 22;");
   /// ```
   pub fn lex(src: &str) -> Self {
      let chars: Vec<char> = src.chars().collect();

      Self {
         source: chars,
         current: 0,
         line_num: 1,
         line_start: 0,
         token_start: 0,
      }
   }

   /// Gets the previously consumed character.
   ///
   /// # Returns
   /// - `char`: The previous character.
   pub fn get_previous(&self) -> char {
      self.source[self.current - 1]
   }

   /// Gets the current character without consuming it.
   ///
   /// # Returns
   /// - `char`: The current character.
   pub fn get_current(&self) -> char {
      self.source[self.current]
   }

   /// Returns the next character without consuming it.
   ///
   /// # Returns
   /// - `char`: The next character.
   pub fn next(&self) -> char {
      if self.is_at_end() {
         return '\0';
      }

      self.source[self.current + 1]
   }

   /// Checks if the scanner is at the end of the source.
   ///
   /// # Returns
   /// - `bool`: True if the scanner is at the end of the source, false otherwise.
   pub fn is_at_end(&self) -> bool {
      return self.current >= self.source.len();
   }

   /// Matches the current character against a provided character
   ///
   /// # Returns
   /// - `bool`: True if the current character matched the provided character, false otherwise.
   pub fn matches(&mut self, expected: char) -> bool {
      if self.is_at_end() || self.get_current() != expected {
         return false;
      }
      self.current += 1;
      return true;
   }

   /// Advances to the next token.
   ///
   /// # Returns
   /// - `char`: The consumed character.
   pub fn advance(&mut self) -> char {
      let c = self.get_current();
      self.current += 1;
      return c;
   }

   /// Skips whitespace-like characters from the source code.
   pub fn skip_whitespace(&mut self) {
      loop {
         if self.is_at_end() {
            break;
         }

         let c = self.get_current();

         if c == ' ' || c == '\r' || c == '\t' {
            self.advance();
         } else if c == '\n' {
            self.line_num += 1;
            self.line_start = self.current + 1;
            self.advance();
         } else if c == '/' && self.next() == '/' {
            self.skip_single_line_comments();
         } else if c == '/' && self.next() == '*' {
            self.skip_block_comments();
         } else {
            break;
         }
      }
   }

   /// Skips single-line comments from the source code.
   fn skip_single_line_comments(&mut self) {
      while !self.is_at_end() && self.get_current() != '\n' {
         self.advance();
      }

      // Reposition the start of the token to
      // be after the comment has ended
      self.token_start = self.current;
   }

   /// Skips block-comments from the source code
   fn skip_block_comments(&mut self) {
      self.advance();
      self.advance();

      while !self.is_at_end() {
         // Recursively skip nested block comments
         if self.get_current() == '/' && self.next() == '*' {
            self.skip_block_comments();

            // break out of the loop if we reached the end of the program
            // before matching the end of this nested block-comment.
            if self.is_at_end() {
               break;
            }
         }

         // Break if we are at the end of the comment.
         if self.get_current() == '*' && self.next() == '/' {
            self.advance();
            self.advance();
            break;
         }

         // Take into account new lines inside block comments
         if self.get_current() == '\n' {
            self.line_num += 1;
         }

         // Skip everything inside the comment
         self.advance();
      }

      // Reposition the start of the token to
      // be after the comment has ended
      self.token_start = self.current;
   }

   /// Makes a string literal.
   pub fn make_string_token(&mut self) -> Token {
      // The opener single or double quote.
      let quote = self.get_previous();

      loop {
         if self.is_at_end() {
            return self.make_error_token("Unterminated string.");
         }

         let prev = self.get_previous();
         let current = self.advance();

         // Take into account new lines inside block strings
         if current == '\n' {
            self.line_num += 1;
            continue;
         }

         // If we reach an unescaped quote, break the loop.
         if current == quote && prev != '\\' {
            break;
         }
      }

      self.make_token(TokenType::STRING)
   }

   /// Generates an identifier token with the current state of the scanner.
   pub fn make_identifier_token(&mut self) -> Token {
      while !self.is_at_end() {
         let c = self.get_current();

         if c.is_alphabetic() || c.is_digit(10) || c == '_' {
            self.advance();
         } else {
            break;
         }
      }

      let id: String = self.source[(self.token_start)..(self.current)]
         .into_iter()
         .collect();
      let tok_type = tokens::make_identifier_type(id.as_str());

      return self.make_token(tok_type);
   }

   /// Generates a token with the current state of the scanner.
   pub fn make_token(&self, token_type: TokenType) -> Token {
      let col_start = match token_type {
         TokenType::EOF => self.current,
         _ => self.token_start - self.line_start,
      };

      let lexeme = match token_type {
         TokenType::EOF => String::from("\0"),
         _ => self.source[(self.token_start)..(self.current)]
            .into_iter()
            .collect(),
      };

      Token {
         line_num: self.line_num,
         column_start: col_start,
         column_end: self.current,
         token_type,
         lexeme,
      }
   }

   /// Generates an error token with the provided message as its lexeme.
   ///
   /// # Parameters
   /// - `message`: A message for the error token. This will be used as the token's lexeme.
   ///
   /// # Returns
   /// - `Token`: The generated error token.
   pub fn make_error_token(&self, message: &str) -> Token {
      Token {
         line_num: self.line_num,
         column_start: self.token_start - self.line_start,
         column_end: self.current,
         token_type: TokenType::ERROR,
         lexeme: String::from(message),
      }
   }
}
