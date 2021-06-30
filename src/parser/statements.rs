use crate::ast::ASTNode;
use crate::ast::ASTNode::*;
use crate::ast::*;
use crate::lexer::tokens::TokenType::*;
use crate::lexer::tokens::{Token, TokenType};
use crate::objects::Object;
use crate::parser::Parser;

impl Parser {
   /// Parses a declaration.
   pub(super) fn parse_declaration(&mut self) -> Option<ASTNode> {
      let decl = if self.matches(&VAR_KW) {
         self.parse_var_declaration().map(VariableDecl)
      } else if self.matches(&CONST_KW) {
         self.parse_const_declaration().map(ConstantDecl)
      } else if self.matches(&FUNC_KW) {
         self.parse_func_declaration().map(FunctionDecl)
      } else if self.matches(&CLASS_KW) {
         self.parse_class_declaration()
      } else {
         self.parse_statement()
      };

      if self.is_in_panic {
         self.synchronize();
      }

      decl
   }

   /// Parses a statement.
   fn parse_statement(&mut self) -> Option<ASTNode> {
      if self.matches(&L_CURLY) {
         self.parse_block()
      } else if self.matches(&IF_KW) {
         self.parse_if_statement()
      } else if self.matches(&WHILE_KW) {
         self.parse_while_statement()
      } else if self.matches(&FOR_KW) {
         self.parse_for_statement()
      } else if self.matches(&BREAK_KW) {
         let tok = self.previous.clone();
         self.consume(&SEMICOLON, "Expected a ';' after the 'break' keyword.");

         Some(LoopBranch(LoopBranchStmtNode {
            token: tok,
            is_break: true,
         }))
      } else if self.matches(&CONTINUE_KW) {
         let tok = self.previous.clone();
         self.consume(&SEMICOLON, "Expected a ';' after the 'continue' keyword.");

         Some(LoopBranch(LoopBranchStmtNode {
            token: tok,
            is_break: false,
         }))
      } else if self.matches(&RETURN_KW) {
         self.parse_return_stmt()
      } else {
         self.parse_expression_statement()
      }
   }

   /// Parses an expression statement.
   fn parse_expression_statement(&mut self) -> Option<ASTNode> {
      let opr = self.previous.clone();
      let expr = self.parse_expression();

      self.consume(&SEMICOLON, "Expected a ';' after the expression.");

      Some(ExpressionStmt(ExpressionStmtNode {
         child: match expr {
            Some(t) => Box::new(t),
            None => return None, // Could not create expression to print
         },
         pos: (opr.line_num, opr.column_start),
      }))
   }

   /// Parses a block statement.
   fn parse_block(&mut self) -> Option<ASTNode> {
      let mut body: Vec<ASTNode> = vec![];

      while !self.check(&R_CURLY) && !self.check(&EOF) {
         match self.parse_declaration() {
            Some(val) => body.push(val),
            // Report parse error if node has None value
            None => return None,
         }
      }

      self.consume(&R_CURLY, "Expected a matching '}' for the block statement.");

      Some(BlockStmt(BlockNode {
         body,
         end_of_block: self.previous.clone(),
      }))
   }

   /// Parses a variable declaration.
   fn parse_var_declaration(&mut self) -> Option<VariableDeclNode> {
      let mut declarations: Vec<Token> = Vec::new();

      // Gets at least one variable name, or a list of
      // names separated by a comma
      self.consume(&IDENTIFIER, "Expected variable name.");

      declarations.push(self.previous.clone());

      while self.matches(&COMMA) {
         self.consume(&IDENTIFIER, "Expected variable name.");

         declarations.push(self.previous.clone());
      }

      // Gets the variable's value.
      let initializer = if self.matches(&EQUALS) {
         match self.parse_expression() {
            Some(val) => val,
            None => return None, // Could not create value for variable
         }
      } else {
         ASTNode::Literal(LiteralExprNode {
            value: Object::Null,
            token: self.previous.clone(),
         })
      };

      // Requires a semicolon at the end of the declaration if the declaration
      // was not a block (e.g., when assigning a lambda function to a variable).
      if !self.previous.token_type.type_match(&R_CURLY) {
         self.consume(&SEMICOLON, "Expected a ';' after the variable declaration.");
      }

      // However, if there is a semicolon after a curly brace, then we consume it
      if self.previous.token_type.type_match(&R_CURLY) && self.check(&SEMICOLON) {
         self.advance();
      }

      Some(VariableDeclNode {
         identifiers: declarations,
         value: Box::new(initializer),
      })
   }

   /// Parses a constant declaration.
   fn parse_const_declaration(&mut self) -> Option<ConstantDeclNode> {
      self.consume(&IDENTIFIER, "Expected a name for the constant declaration.");

      let name = self.previous.clone();

      self.consume(&EQUALS, "Constants must be initialized upon declaration.");

      let initializer = match self.parse_expression() {
         Some(val) => val,
         None => return None, // Could not create value for variable
      };

      // Requires a semicolon at the end of the declaration if the declaration
      // was not a block (e.g., when assigning a lambda function to a constant).
      if !self.previous.token_type.type_match(&R_CURLY) {
         self.consume(&SEMICOLON, "Expected a ';' after the constant declaration.");
      }

      // However, if there is a semicolon after a curly brace, then we consume it
      if self.previous.token_type.type_match(&R_CURLY) && self.check(&SEMICOLON) {
         self.advance();
      }

      Some(ConstantDeclNode {
         name,
         value: Box::new(initializer),
      })
   }

   /// Parses an `if` statement.
   fn parse_if_statement(&mut self) -> Option<ASTNode> {
      let then_tok = self.previous.clone();

      let condition = match self.parse_expression() {
         Some(val) => val,
         None => return None, // Could not create condition for if-statement
      };

      let then_branch;
      if let R_PARENTHESIS = self.previous.token_type {
         then_branch = match self.parse_statement() {
            Some(val) => val,
            None => return None, // Could not create then branch
         };
      } else {
         self.consume(&L_CURLY, "Expected '{' after the 'if' condition.");

         then_branch = match self.parse_block() {
            Some(val) => val,
            None => return None, // Could not create then branch
         };
      }

      let mut else_branch = None;
      let mut else_tok = None;
      if self.matches(&ELSE_KW) {
         else_tok = Some(self.previous.clone());

         else_branch = match self.parse_statement() {
            Some(val) => Some(val),
            None => return None, // Could not create else branch
         };
      }

      Some(IfStmt(IfStmtNode {
         condition: Box::new(condition),
         then_token: then_tok,
         then_branch: Box::new(then_branch),
         else_branch: Box::new(else_branch),
         else_token: else_tok,
      }))
   }

   /// Parses a `while` statement.
   fn parse_while_statement(&mut self) -> Option<ASTNode> {
      let tok = self.previous.clone();

      let condition = match self.parse_expression() {
         Some(val) => val,
         None => return None, // Could not create condition for while-loop
      };

      let body;
      if let R_PARENTHESIS = self.previous.token_type {
         body = match self.parse_statement() {
            Some(val) => val,
            None => return None, // Could not create then branch
         };
      } else {
         self.consume(&L_CURLY, "Expected '{' after the 'while' condition.");

         body = match self.parse_block() {
            Some(val) => val,
            None => return None, // Could not create then branch
         };
      }

      Some(WhileStmt(WhileStmtNode {
         token: tok,
         condition: Box::new(condition),
         body: Box::new(body),
      }))
   }

   /// Parses a `for-in` statement.
   fn parse_for_statement(&mut self) -> Option<ASTNode> {
      let token = self.previous.clone();

      let mut has_parenthesis = false;
      if self.matches(&L_PAREN) {
         has_parenthesis = true;
      }

      // For-loops must have either the `let` or `await` keyword before the loop's variable, but
      // not both. Here, in the future, we would check which keyword it is and define the type
      // of for-loop we are parsing based on which keyword is present.
      self.consume(&VAR_KW, "Expected the 'let' keyword before the identifier.");

      let id = match self.parse_primary() {
         Some(val) => match val {
            ASTNode::Identifier(i) => i,
            _ => {
               self.error_at_current("Expected an identifier name.");
               return None;
            }
         },
         None => return None, // Could not parse an identifier for loop
      };

      self.consume(&IN_KW, "Expected the 'in' keyword after the identifier.");

      let iterator = match self.parse_expression() {
         Some(expr) => Box::new(expr),
         None => return None, // Could not parse an iterator expression
      };

      let body: Vec<ASTNode>;
      if has_parenthesis {
         self.consume(
            &R_PARENTHESIS,
            "Expected a matching ')' after the 'for-in' iterator.",
         );

         body = match self.parse_statement() {
            Some(val) => match val {
               ASTNode::BlockStmt(block) => block.body,
               _ => vec![val],
            },
            None => return None, // Could not create then branch
         };
      } else {
         self.consume(&L_CURLY, "Expected '{' after the 'for-in' iterator.");

         body = match self.parse_block() {
            Some(val) => match val {
               ASTNode::BlockStmt(block) => block.body,
               _ => unreachable!("Should have parsed a block."),
            },
            None => return None, // Could not create then branch
         };
      }

      Some(ForStmt(ForStmtNode {
         token,
         id,
         iterator,
         body,
      }))
   }

   /// Parses a function declaration.
   fn parse_func_declaration(&mut self) -> Option<FunctionDeclNode> {
      self.consume(
         &IDENTIFIER,
         "Expected an identifier for the function declaration.",
      );

      let name = self.previous.clone();
      self.consume(&L_PAREN, "Expected '(' after function name.");

      let mut params: Vec<Parameter> = vec![];
      let mut min_arity: u8 = 0;
      let mut max_arity: u8 = 0;

      while !self.matches(&R_PARENTHESIS) {
         if params.len() >= 255 {
            self.error_at_current("Can't have more than 255 parameters.");
            return None;
         }

         match self.parse_parameter() {
            Some(p) => {
               if !params.is_empty() && !p.is_optional && params.last().unwrap().is_optional {
                  self.error_at_token(
                     &params.last().unwrap().name.clone(),
                     "Optional and named parameters must be declared after all required parameters.",
                  );
                  return None;
               }

               max_arity += 1;

               if !p.is_optional {
                  min_arity += 1
               }

               params.push(p);
            }
            None => return None, // Could not parse the parameter
         }

         if !self.matches(&R_PARENTHESIS) {
            self.consume(&COMMA, "Expected a ',' between the parameter declarations.");
         } else {
            break;
         }
      }

      self.consume(&L_CURLY, "Expected '{' for the function body.");

      return Some(FunctionDeclNode {
         name,
         params,
         arity: (min_arity, max_arity),
         body: match self.parse_block() {
            Some(node) => match node {
               BlockStmt(b) => b.body,
               _ => unreachable!("Should have parsed a block statement."),
            },
            None => return None,
         },
      });
   }

   /// Parses a parameter declaration.
   fn parse_parameter(&mut self) -> Option<Parameter> {
      self.consume(&IDENTIFIER, "Expected a parameter name.");

      let name = self.previous.clone();

      if self.matches(&QUESTION) {
         return Some(Parameter {
            name,
            is_optional: true,
            default: None,
         });
      }

      if self.matches(&COLON_EQUALS) {
         return Some(Parameter {
            name,
            is_optional: true,
            default: match self.parse_expression() {
               Some(x) => Some(Box::new(x)),
               None => return None, // Could not compile default value for parameter
            },
         });
      }

      Some(Parameter {
         name,
         is_optional: false,
         default: None,
      })
   }

   /// Parses a `return` statement.
   fn parse_return_stmt(&mut self) -> Option<ASTNode> {
      let tok = self.previous.clone();

      // Compiles the return expression
      if !self.matches(&SEMICOLON) {
         let expr = match self.parse_expression() {
            Some(val) => val,
            // Report parse error if node has None value
            None => return None,
         };

         self.consume(&SEMICOLON, "Expected a ';' after the expression.");

         return Some(ReturnStmt(ReturnStmtNode {
            token: tok,
            value: Some(Box::new(expr)),
         }));
      }

      Some(ReturnStmt(ReturnStmtNode {
         token: tok,
         value: None,
      }))
   }

   /// Parses a `class` declaration statement.
   fn parse_class_declaration(&mut self) -> Option<ASTNode> {
      self.consume(&IDENTIFIER, "Expected an identifier for the class declaration.");
      let name = self.previous.clone();

      self.consume(&L_CURLY, "Expected '{' for the class body.");
      let mut members: Vec<ClassMemberDeclNode> = vec![];

      while !self.matches(&TokenType::R_CURLY) {
         let member_type = if self.matches(&FUNC_KW) {
            match self.parse_func_declaration() {
               Some(decl) => ClassMemberDecl::Method(decl),
               None => return None, // Could not parse method
            }
         } else if self.matches(&VAR_KW) {
            match self.parse_var_declaration() {
               Some(decl) => ClassMemberDecl::Var(decl),
               None => return None, // Could not parse variable field
            }
         } else if self.matches(&CONST_KW) {
            match self.parse_const_declaration() {
               Some(decl) => ClassMemberDecl::Const(decl),
               None => return None, // Could not parse constant field
            }
         } else {
            if self.check(&EOF) {
               self.error_at_current("Unexpected end of file while parsing class body.")
            } else {
               self.error_at_current("Unexpected token.");
            }
            return None;
         };

         members.push(ClassMemberDeclNode { member_type });
      }

      Some(ClassDecl(ClassDeclNode { name, members }))
   }
}
