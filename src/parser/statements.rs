use super::Parser;
use std::vec;

use crate::{
    ast::ASTNode::*,
    ast::*,
    lexer::tokens::{Token, TokenType::*},
    objects::Object,
};

impl Parser {
    /// Parses a declaration as specified in the grammar.bnf file.
    ///
    /// ## Returns
    /// `Option<ASTNode>` – The declaration's AST node.
    pub(super) fn parse_declaration(&mut self) -> Option<ASTNode> {
        let decl = if self.matches(&LET_KW) {
            self.parse_var_declaration()
        } else if self.matches(&CONST_KW) {
            self.parse_const_declaration()
        } else if self.matches(&FUNC_KW) {
            self.parse_func_declaration()
        } else if self.matches(&ENUM_KW) {
            todo!("Implement enum declarations")
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

    /// Parses a statement as specified in the grammar.bnf file.
    ///
    /// ## Returns
    /// `Option<ASTNode>` – The statement's AST node.
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
            self.consume(&SEMICOLON, "Expected ';' after break keyword.");

            return Some(LoopBranch(LoopBranchStmtNode {
                token: tok.clone(),
                is_break: true,
            }));
        } else if self.matches(&CONTINUE_KW) {
            let tok = self.previous.clone();
            self.consume(&SEMICOLON, "Expected ';' after continue keyword.");

            return Some(LoopBranch(LoopBranchStmtNode {
                token: tok.clone(),
                is_break: false,
            }));
        } else if self.matches(&RETURN_KW) {
            self.parse_return_stmt()
        } else {
            self.parse_expression_statement()
        }
    }

    /// Parses an expression statement as specified in the grammar.bnf file.
    ///
    /// ## Returns
    /// `Option<ASTNode>` – The expression statement's AST node.
    fn parse_expression_statement(&mut self) -> Option<ASTNode> {
        let opr = self.previous.clone();
        let expr = self.parse_expression();

        self.consume(&SEMICOLON, "Expected ';' after expression.");

        return Some(ExpressionStmt(ExpressionStmtNode {
            child: match expr {
                Some(t) => Box::new(t),
                None => return None, // Could not create expression to print
            },
            pos: (opr.line_num, opr.column_num),
        }));
    }

    /// Parses a block statement.
    ///
    /// ## Returns
    /// `Option<ASTNode>` – The block statement's AST node.
    fn parse_block(&mut self) -> Option<ASTNode> {
        let mut body: Vec<ASTNode> = vec![];

        while !self.check(&R_CURLY) && !self.check(&EOF) {
            match self.parse_declaration() {
                Some(val) => body.push(val),
                // Report parse error if node has None value
                None => return None,
            }
        }

        self.consume(&R_CURLY, "Expect '}' after block.");

        return Some(BlockStmt(BlockNode {
            body,
            end_of_block: self.previous.clone(),
            // If the block is, indeed, a function body,
            // the `parse_func_declaration(...)` method
            // will change this to true.
            is_func_body: false,
        }));
    }

    /// Parses a variable declaration as specified in the grammar.bnf file.
    ///
    /// Returns
    /// * `Option<ASTNode>` – A variable declaration AST node.
    fn parse_var_declaration(&mut self) -> Option<ASTNode> {
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
            self.consume(&SEMICOLON, "Expected ';' after variable declaration.");
        }

        // But if there is a semicolon after a curly brace, then we consume it
        if self.previous.token_type.type_match(&R_CURLY) && self.check(&SEMICOLON) {
            self.advance();
        }

        return Some(ASTNode::VariableDecl(VariableDeclNode {
            identifiers: declarations,
            value: Box::new(initializer.clone()),
        }));
    }

    /// Parses a variable declaration as specified in the grammar.bnf file.
    ///
    /// Returns
    /// * `Option<ASTNode>` – A variable declaration AST node.
    fn parse_const_declaration(&mut self) -> Option<ASTNode> {
        self.consume(&IDENTIFIER, "Expected variable name.");

        let name = self.previous.clone();

        self.consume(&EQUALS, "Constants must have a value.");

        let initializer = match self.parse_expression() {
            Some(val) => val,
            None => return None, // Could not create value for variable
        };

        // Requires a semicolon at the end of the declaration if the declaration
        // was not a block (e.g., when assigning a lambda function to a constant).
        if !self.previous.token_type.type_match(&R_CURLY) {
            self.consume(&SEMICOLON, "Expected ';' after constant declaration.");
        }

        // But if there is a semicolon after a curly brace, then we consume it
        if self.previous.token_type.type_match(&R_CURLY) && self.check(&SEMICOLON) {
            self.advance();
        }

        return Some(ASTNode::ConstantDecl(ConstantDeclNode {
            name,
            value: Box::new(initializer.clone()),
        }));
    }

    /// Parses an if statement as specified in the grammar.bnf file.
    ///
    /// Returns
    /// * `Option<ASTNode>` – An if statement AST node.
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
            self.consume(&L_CURLY, "Expected '{' after 'if' condition.");

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

        return Some(IfStmt(IfStmtNode {
            condition: Box::new(condition),
            then_token: then_tok,
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
            else_token: else_tok,
        }));
    }

    /// Parses a while statement as specified in the grammar.bnf file.
    ///
    /// Returns
    /// * `Option<ASTNode>` – A while statement AST node.
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
            self.consume(&L_CURLY, "Expected '{' after 'while' condition.");

            body = match self.parse_block() {
                Some(val) => val,
                None => return None, // Could not create then branch
            };
        }

        return Some(WhileStmt(WhileStmtNode {
            token: tok,
            condition: Box::new(condition),
            body: Box::new(body),
        }));
    }

    /// Parses a for statement as specified in the grammar.bnf file.
    ///
    /// Returns
    /// * `Option<ASTNode>` – A for statement AST node.
    fn parse_for_statement(&mut self) -> Option<ASTNode> {
        let token = self.previous.clone();

        let mut has_parenthesis = false;
        if self.matches(&L_PAREN) {
            has_parenthesis = true;
        }

        // For-loops must have either the `let` or `await` keyword before the loop's variable,
        // but not both. Here, in the future, we would check which keyword it is and define
        // the type of for-loop we are parsing based on which keyword is present.
        self.consume(&LET_KW, "Expected 'let' before for-loop variable.");

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

        self.consume(&IN_KW, "Expected 'in' after identifier.");

        let iterator = match self.parse_expression() {
            Some(expr) => Box::new(expr),
            None => return None, // Could not parse an iterator expression
        };

        let body: Vec<ASTNode>;
        if has_parenthesis {
            self.consume(&R_PARENTHESIS, "Expected ')' after 'for' iterator.");

            body = match self.parse_statement() {
                Some(val) => match val {
                    ASTNode::BlockStmt(block) => block.body,
                    _ => vec![val],
                },
                None => return None, // Could not create then branch
            };
        } else {
            self.consume(&L_CURLY, "Expected '{' after 'for' iterator.");

            body = match self.parse_block() {
                Some(val) => match val {
                    ASTNode::BlockStmt(block) => block.body,
                    _ => unreachable!("Should have parsed a block."),
                },
                None => return None, // Could not create then branch
            };
        }

        return Some(ForStmt(ForStmtNode {
            token,
            id,
            iterator,
            body,
        }));
    }

    /// Parses a function declaration as specified in the grammar.bnf file.
    ///
    /// Returns
    /// * `Option<ASTNode>` – A function declaration AST node.
    fn parse_func_declaration(&mut self) -> Option<ASTNode> {
        self.consume(&IDENTIFIER, "Expected a function name.");

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
                    if params.len() > 0 && !p.is_optional && params.last().unwrap().is_optional {
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
                self.consume(&COMMA, "Expected ',' after parameter.");
            } else {
                break;
            }
        }

        self.consume(&L_CURLY, "Expected '{' braces before function body.");

        let mut body = match self.parse_block() {
            Some(b) => b,
            None => return None,
        };

        // Checks if the last statement in the function's block is a return statement.
        // if it is, we don't emit an extra return at compile time.
        let ends_with_return = match &mut body {
            BlockStmt(b) => {
                b.is_func_body = true;

                if b.body.len() > 0 {
                    if let ReturnStmt(_) = b.body.last().unwrap() {
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            _ => unreachable!("Should have parsed a block statement."),
        };

        return Some(FunctionDecl(FunctionDeclNode {
            name,
            params,
            min_arity,
            max_arity,
            body: Box::new(body),
            ends_with_return,
        }));
    }

    /// Parses a parameter declaration as specified in the grammar.bnf file.
    ///
    /// Returns
    /// * `Option<ASTNode>` – A parameter declaration AST node.
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

    /// Parses a return statement as specified in the grammar.bnf file.
    ///
    /// Returns
    /// * `Option<ASTNode>` – A return statement AST node.
    fn parse_return_stmt(&mut self) -> Option<ASTNode> {
        let tok = self.previous.clone();

        // Compiles the return expression
        if !self.matches(&SEMICOLON) {
            let expr = match self.parse_expression() {
                Some(val) => val,
                // Report parse error if node has None value
                None => return None,
            };

            self.consume(&SEMICOLON, "Expected ';' after break keyword.");

            return Some(ReturnStmt(ReturnStmtNode {
                token: tok,
                value: Some(Box::new(expr)),
            }));
        }

        return Some(ReturnStmt(ReturnStmtNode {
            token: tok,
            value: None,
        }));
    }

    /// Parses a class declaration statement as specified in the grammar.bnf file.
    ///
    /// Returns
    /// * `Option<ASTNode>` – A class declaration statement AST node.
    fn parse_class_declaration(&mut self) -> Option<ASTNode> {
        self.consume(&IDENTIFIER, "Expected a class name.");
        let name = self.previous.clone();

        self.consume(&L_CURLY, "Expected '{' before class body.");
        self.consume(&R_CURLY, "Expected '}' after class body.");

        return Some(ClassDecl(ClassDeclNode { name }));
    }
}
