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
   line: usize,
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
         line: 1,
         line_start: 0,
         token_start: 0,
      }
   }

   /// Gets the previously consumed character.
   ///
   /// # Returns
   /// - `char`: The previous character.
   pub fn previous(&self) -> char {
      self.source[self.current - 1]
   }

   /// Gets the current character without consuming it.
   ///
   /// # Returns
   /// - `char`: The current character.
   pub fn get_current(&self) -> char {
      // TODO: What can you do so that this check is no longer needed?
      // NOTE: At the moment, if this check is removed, Rust will panic
      // because at some point in the program (I am not sure where), the
      // lexer tries to get the (N + 1)th element of the vector, causing
      // an index-out-of-bounds error.
      if self.is_at_end() {
         return '\0';
      }

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
         let c = self.get_current();

         if c == ' ' || c == '\r' || c == '\t' {
            self.advance();
         } else if c == '\n' {
            self.line += 1;
            self.line_start = self.current + 1;
            self.advance();
         } else if c == '/' && (self.next() == '/' || self.next() == '*') {
            self.skip_comments();
         } else {
            break;
         }
      }
   }

   /// Skips single-line and block comments from the source code.
   /// TODO: Allow nesting block comments
   pub fn skip_comments(&mut self) {
      // single-line comments
      if self.get_current() == '/' && self.next() == '/' {
         while self.get_current() != '\n' && !self.is_at_end() {
            self.advance();
         }
      }

      // block comments
      if self.get_current() == '/' && self.next() == '*' {
         self.advance();
         self.advance();

         loop {
            // Break if we are at the end of the comment.
            if self.get_current() == '*' && self.next() == '/' {
               self.advance();
               self.advance();
               break;
            }

            // Break if we have reached the end of the program
            if self.is_at_end() {
               break;
            };

            // Take into account new lines inside block comments
            if self.get_current() == '\n' {
               self.line += 1;
            }

            // Skip everything inside the comment
            self.advance();
         }
      }

      // Reposition the start of the token to
      // be after the comment has ended
      self.token_start = self.current;
   }

   /// Makes a string literal.
   ///
   /// # Returns
   /// - `Token`: A string token.
   pub fn make_string_token(&mut self) -> Token {
      // The opener single or double quote.
      let quote = self.previous();

      // Keep consuming characters until there is an unescaped quote or
      // the program reaches the end of the source file.
      while (self.get_current() != quote || (self.get_current() == quote && self.previous() == '\\'))
         && !self.is_at_end()
      {
         self.advance();
      }

      if self.is_at_end() {
         return self.make_error_token("Unterminated string.");
      }

      // The closing quotes.
      self.advance();

      return self.make_token(TokenType::STRING);
   }

   /// Generates an identifier token with the current state of the scanner.
   ///
   /// # Returns
   /// - `Token`: An identifier token.
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

   /// Generates a token with the current state of the scanner
   /// # Parameters
   /// - `tok_type`: The type of the token to generate.
   ///
   /// # Returns
   /// - `Token`: The generated token (of any type).
   pub fn make_token(&self, tok_type: TokenType) -> Token {
      Token {
         line_num: self.line,
         column_num: if tok_type.clone() as u8 != TokenType::EOF as u8 {
            self.token_start - self.line_start
         } else {
            self.current
         },
         token_type: tok_type.clone(),
         // If the token is the EOF token, then the lexeme becomes the null terminator.
         // It is okay to use the null terminator for the EOF value because the EOF Token's
         // lexeme is never used for anything.
         lexeme: if tok_type as u8 != TokenType::EOF as u8 {
            self.source[(self.token_start)..(self.current)]
               .into_iter()
               .collect()
         } else {
            String::from("\0")
         },
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
         line_num: self.line,
         column_num: self.token_start - self.line_start,
         token_type: TokenType::ERROR,
         lexeme: String::from(message),
      }
   }
}
