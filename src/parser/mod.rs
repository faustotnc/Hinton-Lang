use crate::ast::{ASTNode, ModuleNode};
use crate::errors::ErrorReport;
use crate::lexer::tokens::TokenType::*;
use crate::lexer::tokens::{Token, TokenType};
use crate::lexer::Lexer;

// Submodules
mod expressions;
mod statements;

/// Represents Hinton's parser, which converts source text into
/// an Abstract Syntax Tree representation of the program.
pub struct Parser {
   /// The lexer used in this parser.
   lexer: Lexer,
   /// The previously consumed token.
   previous: Token,
   /// The current token (just consumed).
   current: Token,
   /// Whether the parser is in error-recovery mode or not.
   is_in_panic: bool,
   /// A list of reported errors generated while parsing.
   errors: Vec<ErrorReport>,
}

impl Parser {
   /// Parses a string of source test into a Hinton AST.
   ///
   /// # Parameters
   /// - `src`: The source string for the program.
   ///
   /// # Returns
   /// - `Ok(ASTNode)`: The generated abstract syntax tree.
   /// - `Err(Vec<ErrorReport>)`: A list of parsing errors.
   pub fn parse(src: &str) -> Result<ASTNode, Vec<ErrorReport>> {
      // Initialize the compiler
      let mut parser = Parser {
         lexer: Lexer::lex(src),
         previous: Token {
            line_num: 0,
            column_start: 0,
            column_end: 0,
            token_type: __INIT_PARSER__,
            lexeme: String::from(""),
         },
         current: Token {
            line_num: 0,
            column_start: 0,
            column_end: 0,
            token_type: __INIT_PARSER__,
            lexeme: String::from(""),
         },
         is_in_panic: false,
         errors: vec![],
      };

      let mut program = ModuleNode { body: vec![] };

      // Start compiling the chunk
      parser.advance();
      while !parser.matches(&EOF) {
         match parser.parse_declaration() {
            Some(val) => program.body.push(val),
            None => {
               // If there was an error, continue parsing
               // to catch other errors in the program, but
               // the AST will (of course) not be usable.
            }
         }
      }

      return if parser.errors.len() == 0 {
         Ok(ASTNode::Module(program))
      } else {
         Err(parser.errors)
      };
   }

   /// Checks that the current token matches the tokenType provided.
   ///
   /// # Parameters
   /// - `type` The tokenType we expect to match with the current token.
   ///
   /// # Results
   /// - `bool`: True if the current token matches the given token type false otherwise.
   fn check(&mut self, tok_type: &TokenType) -> bool {
      self.get_current_tok_type().type_match(tok_type)
   }

   /// Checks that the current token matches the tokenType provided.
   /// If the tokens match, the current token gets consumed, and the function returns true.  
   /// Otherwise, if the tokens do not match, the token is not consumed, and the function
   /// returns false.
   ///
   /// # Parameters
   /// - `tok_type` The tokenType we expect to match with the current token.
   ///
   /// # Returns
   /// `bool`: True if the tokens match, false otherwise.
   fn matches(&mut self, tok_type: &TokenType) -> bool {
      return if self.check(tok_type) {
         self.advance();
         true
      } else {
         false
      };
   }

   /// Advances the parser to the next token.
   fn advance(&mut self) {
      self.previous = self.current.clone();

      loop {
         self.current = self.lexer.next_token();

         match &self.current.token_type {
            ERROR => self.error_at_current("Unexpected token."),
            _ => break,
         }
      }
   }

   /// Consumes the current token only if it is of a given type. If the token does not match the
   /// type, emits a compiler error.
   ///
   /// # Parameters
   /// - `tok_type`: The expected type of the token to consume.
   /// - `message`: The error message to be displayed if the current token does not match the
   /// provided type.
   fn consume(&mut self, tok_type: &TokenType, message: &str) {
      if self.check(tok_type) {
         self.advance();
         return;
      }

      if let SEMICOLON = tok_type {
         self.error_at_previous(message);
      } else {
         self.error_at_current(message);
      }
   }

   /// Gets the type of the current token.
   ///
   /// # Returns
   /// `TokenType`: The type of the current token.
   fn get_current_tok_type(&self) -> &TokenType {
      &self.current.token_type
   }

   /// Gets the type of the previous token.
   ///
   /// # Returns
   /// `TokenType`: The type of the previous token.
   fn get_previous_tok_type(&self) -> TokenType {
      self.previous.token_type.clone()
   }

   /// Emits a compiler error from the current token.
   ///
   /// # Parameters
   /// - `message`: The error message to display.
   fn error_at_current(&mut self, message: &str) {
      self.error_at_token(&self.current.clone(), message);
   }

   /// Emits a compiler error from the previous token.
   ///
   /// # Parameters
   /// - `message`: The error message to display.
   fn error_at_previous(&mut self, message: &str) {
      self.error_at_token(&self.previous.clone(), message);
   }

   /// Emits a compiler error from the given token.
   ///
   /// # Parameters
   /// - `tok`: The token that caused the error.
   /// - `message`: The error message to display.
   fn error_at_token(&mut self, tok: &Token, message: &str) {
      if self.is_in_panic {
         return;
      }
      self.is_in_panic = true;

      // Construct the error message.
      let msg = format!(
         "\x1b[31;1mSyntaxError\x1b[0m\x1b[1m at [{}:{}]: {}\x1b[0m",
         tok.line_num, tok.column_start, message
      );

      // Push the error to the list
      self.errors.push(ErrorReport {
         line: tok.line_num,
         column: tok.column_start,
         lexeme_len: tok.lexeme.len(),
         message: msg,
      });
   }

   /// Synchronizes the compiler when it has found an error.
   /// This method helps minimize the number of cascading errors the compiler emits
   /// when it finds a parsing error. Once it reaches a synchronization point – like
   /// a keyword for a statement – it stops emitting errors.
   fn synchronize(&mut self) {
      self.is_in_panic = false;

      while !self.get_current_tok_type().type_match(&EOF) {
         if let SEMICOLON = self.get_previous_tok_type() {
            return;
         }

         match self.get_current_tok_type() {
            CLASS_KW | FUNC_KW | LET_KW | FOR_KW | IF_KW | WHILE_KW | RETURN_KW => {
               return;
            }

            _ => {}
         }

         self.advance();
      }
   }
}
