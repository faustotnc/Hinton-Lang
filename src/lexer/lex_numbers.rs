use crate::lexer::tokens::Token;
use crate::lexer::tokens::TokenType::*;
use crate::lexer::Lexer;

impl Lexer {
   /// Makes a numeric literal. This includes Binary, Octal, Decimal,
   /// Floating-Point, and Hexadecimal numbers.
   ///
   /// # Returns
   /// - `Token`: A numeric token (integer, float, binary, octal, or hex).
   pub(super) fn make_numeric_token(&mut self) -> Token {
      // Support for hexadecimal integers
      // Hexadecimal literals are converted to integer literals during compilation
      if self.get_previous() == '0' && (self.get_current() == 'x' || self.get_current() == 'X') {
         self.advance(); // consumes the "x"
         self.advance_numeric_digit(16); // Consume digit character in base-16
         return self.make_token(HEXADECIMAL);
      }

      // Support for octal integers
      // Octal literals are converted to integer literals during compilation
      if self.get_previous() == '0' && (self.get_current() == 'o' || self.get_current() == 'O') {
         self.advance(); // consumes the 'o'
         self.advance_numeric_digit(8); // Consume digit character in base-8
         return self.make_token(OCTAL);
      }

      // Support for binary integers
      // Binary literals are converted to integer literals during compilation
      if self.get_previous() == '0' && (self.get_current() == 'b' || self.get_current() == 'B') {
         self.advance(); // consumes the 'b'
         self.advance_numeric_digit(2); // Consume digit character in base-2
         return self.make_token(BINARY);
      }

      // Checks whether the numeric token started with a dot (to correctly mark it as a float).
      let started_with_dot = self.get_previous() == '.';
      self.advance_numeric_digit(10); // Consume digit character in base-10

      // Look for a fractional part (only for floats that do not start with a dot).
      if !started_with_dot && self.get_current() == '.' && self.next().is_digit(10) {
         self.advance(); // Consume the ".".
         self.advance_numeric_digit(10); // Consume digit character in base-10
         return self.make_token(FLOAT);
      }

      if started_with_dot {
         self.make_token(FLOAT)
      } else {
         self.make_token(INTEGER)
      }
   }

   /// Consumes digit characters of the given radix base.
   ///
   /// # Arguments
   /// - `radix`: The base of the expected digit.
   fn advance_numeric_digit(&mut self, radix: u32) {
      while !self.is_at_end() && self.get_current().is_digit(radix)
         || (self.get_current() == '_' && self.get_previous() != '_')
      {
         self.advance();
      }
   }
}
