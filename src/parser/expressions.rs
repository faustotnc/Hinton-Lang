use super::Parser;
use crate::{
    ast::ASTNode::*,
    ast::*,
    lexer::tokens::{Token, TokenType::*},
    objects::Object,
};

// ========= Precedence Table for Expressions (Low to High) =========
// Assignment:      <identifier> (=, +=, -=, /=, etc...) <expr>
// Ternary:         <expr> ? <expr> : <expr>
// Nullish:         <expr> ?? <expr>
// Logic Or:        (<expr> || <expr>), (<expr> or <expr>)
// Logic And:       (<expr> && <expr>), (<expr> and <expr>)
// Bitwise Or:      <expr> | <expr>
// Bitwise Xor:     <expr> ^ <expr>
// Bitwise And:     <expr> & <expr>
// Equality:        (<expr> == <expr>), (<expr> equals <expr>)
// Comparison:      <expr> (<, <=, >, >=) <expr>
// Range:           <expr> .. <expr>
// Bitwise Shift:   <expr> (<<, >>) <expr>
// Term:            <expr> (+, -) <expr>
// Factor:          <expr> (*, /, %, mod) <expr>
// Expo:            <expr> ** <expr>
// Unary:           (~, !, -, not) <expr>
// ==================================================================

impl<'a> Parser {
    /// Parses an expression as specified in the grammar.bnf file.
    ///
    /// ## Returns
    /// `Option<ASTNode>` – The expression's AST node.
    pub(super) fn parse_expression(&mut self) -> Option<ASTNode> {
        self.parse_assignment()
    }

    /// Parses an assignment expression as specified in the grammar.bnf file.
    ///
    /// ## Returns
    /// `Option<ASTNode>` – The assignment expression's AST node.
    fn parse_assignment(&mut self) -> Option<ASTNode> {
        let expr = self.parse_ternary_conditional();
        let expr_tok = self.previous.clone();

        if self.matches(&EQUALS)
            || self.matches(&PLUS_EQ)
            || self.matches(&MINUS_EQ)
            || self.matches(&STAR_EQ)
            || self.matches(&SLASH_EQ)
            || self.matches(&EXPO_EQUALS)
            || self.matches(&MOD_EQ)
            || self.matches(&BIT_L_SHIFT_EQ)
            || self.matches(&BIT_R_SHIFT_EQ)
            || self.matches(&BIT_AND_EQ)
            || self.matches(&BIT_XOR_EQ)
            || self.matches(&BIT_OR_EQ)
        {
            let opr = self.previous.clone();

            // Gets the type of reassignment
            let opr_type = match opr.token_type {
                PLUS_EQ => ReassignmentType::Plus,
                MINUS_EQ => ReassignmentType::Minus,
                STAR_EQ => ReassignmentType::Mul,
                SLASH_EQ => ReassignmentType::Div,
                EXPO_EQUALS => ReassignmentType::Expo,
                MOD_EQ => ReassignmentType::Mod,
                BIT_L_SHIFT_EQ => ReassignmentType::ShiftL,
                BIT_R_SHIFT_EQ => ReassignmentType::ShiftR,
                BIT_AND_EQ => ReassignmentType::BitAnd,
                BIT_XOR_EQ => ReassignmentType::Xor,
                BIT_OR_EQ => ReassignmentType::BitOr,
                // Regular re-assignment
                _ => ReassignmentType::None,
            };

            // Gets the value for assignment
            let rhs = match self.parse_expression() {
                Some(t) => t,
                None => return None, // Could not create expression value for assignment
            };

            // Returns the assignment expression of the corresponding type
            return match expr {
                Some(node) => match node {
                    // Variable re-assignment.
                    Identifier(id) => Some(VarReassignment(VarReassignmentExprNode {
                        target: id.token,
                        opr_type,
                        value: Box::new(rhs),
                        pos: (opr.line_num, opr.column_num),
                    })),

                    // Object setter `object.property = value`.
                    ObjectGetter(getter) => Some(ObjectSetter(ObjectSetExprNode {
                        target: getter.target,
                        setter: getter.getter,
                        value: Box::new(rhs),
                        opr_type,
                    })),

                    // The assignment target is not valid
                    _ => {
                        self.error_at_token(&expr_tok, "Invalid assignment target.");
                        None
                    }
                },

                // Could not parse lhs of expression
                None => None,
            };
        }

        return expr;
    }

    /// Parses a ternary conditional expression as specified in the grammar.bnf file.
    ///
    /// ## Returns
    /// `Option<ASTNode>` – The assignment expression's AST node.
    fn parse_ternary_conditional(&mut self) -> Option<ASTNode> {
        let mut expr = self.parse_nullish_coalescing();

        if self.matches(&QUESTION) {
            let true_branch_opr = self.previous.clone();

            let branch_true = match self.parse_expression() {
                Some(t) => t,
                None => return None, // Could not create expression for branch_true
            };

            self.consume(&COLON, "Expected ':' in ternary operator.");

            let false_branch_opr = self.previous.clone();

            let branch_false = match self.parse_expression() {
                Some(t) => t,
                None => return None, // Could not create expression for branch_false
            };

            expr = Some(TernaryConditional(TernaryConditionalNode {
                condition: match expr {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create conditional expression
                },
                true_branch_token: true_branch_opr,
                branch_true: Box::new(branch_true),
                branch_false: Box::new(branch_false),
                false_branch_token: false_branch_opr,
            }));
        }

        return expr;
    }

    /// Parses an '??' (nullish coalescing) expression as specified in the grammar.bnf file.
    ///
    /// ## Returns
    /// `Option<ASTNode>` – The expression's AST node.
    fn parse_nullish_coalescing(&mut self) -> Option<ASTNode> {
        let mut expr = self.parse_logic_or();

        while self.matches(&NULLISH) {
            let opr = self.previous.clone();

            expr = Some(Binary(BinaryExprNode {
                left: match expr {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create lhs of expression
                },
                right: match self.parse_logic_or() {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create rhs of expression
                },
                opr_token: opr,
                opr_type: BinaryExprType::Nullish,
            }));
        }

        return expr;
    }

    /// Parses an 'OR' expression as specified in the grammar.bnf file.
    ///
    /// ## Returns
    /// `Option<ASTNode>` – The expression's AST node.
    fn parse_logic_or(&mut self) -> Option<ASTNode> {
        let mut expr = self.parse_logic_and();

        while self.matches(&LOGIC_OR) {
            let opr = self.previous.clone();

            expr = Some(Binary(BinaryExprNode {
                left: match expr {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create lhs of expression
                },
                right: match self.parse_logic_and() {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create rhs of expression
                },
                opr_token: opr,
                opr_type: BinaryExprType::LogicOR,
            }));
        }

        return expr;
    }

    /// Parses an 'AND' expression as specified in the grammar.bnf file.
    ///
    /// ## Returns
    /// `Option<ASTNode>` – The expression's AST node.
    fn parse_logic_and(&mut self) -> Option<ASTNode> {
        let mut expr = self.parse_bitwise_or();

        while self.matches(&LOGIC_AND) {
            let opr = self.previous.clone();

            expr = Some(Binary(BinaryExprNode {
                left: match expr {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create lhs of expression
                },
                right: match self.parse_bitwise_or() {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create rhs of expression
                },
                opr_token: opr,
                opr_type: BinaryExprType::LogicAND,
            }));
        }

        return expr;
    }

    /// Parses a 'BITWISE OR' expression as specified in the grammar.bnf file.
    ///
    /// ## Returns
    /// `Option<ASTNode>` – The expression's AST node.
    fn parse_bitwise_or(&mut self) -> Option<ASTNode> {
        let mut expr = self.parse_bitwise_xor();

        while self.matches(&BIT_OR) {
            let opr = self.previous.clone();

            expr = Some(Binary(BinaryExprNode {
                left: match expr {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create lhs of expression
                },
                right: match self.parse_bitwise_xor() {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create rhs of expression
                },
                opr_token: opr,
                opr_type: BinaryExprType::BitwiseOR,
            }));
        }

        return expr;
    }

    /// Parses a 'BITWISE XOR' expression as specified in the grammar.bnf file.
    ///
    /// ## Returns
    /// `Option<ASTNode>` – The expression's AST node.
    fn parse_bitwise_xor(&mut self) -> Option<ASTNode> {
        let mut expr = self.parse_bitwise_and();

        while self.matches(&BIT_XOR) {
            let opr = self.previous.clone();

            expr = Some(Binary(BinaryExprNode {
                left: match expr {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create lhs of expression
                },
                right: match self.parse_bitwise_and() {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create rhs of expression
                },
                opr_token: opr,
                opr_type: BinaryExprType::BitwiseXOR,
            }));
        }

        return expr;
    }

    /// Parses a 'BITWISE AND' expression as specified in the grammar.bnf file.
    ///
    /// ## Returns
    /// `Option<ASTNode>` – The expression's AST node.
    fn parse_bitwise_and(&mut self) -> Option<ASTNode> {
        let mut expr = self.parse_equality();

        while self.matches(&BIT_AND) {
            let opr = self.previous.clone();

            expr = Some(Binary(BinaryExprNode {
                left: match expr {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create lhs of expression
                },
                right: match self.parse_equality() {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create rhs of expression
                },
                opr_token: opr,
                opr_type: BinaryExprType::BitwiseAND,
            }));
        }

        return expr;
    }

    /// Parses an equality expression as specified in the grammar.bnf file.
    ///
    /// ## Returns
    /// `Option<ASTNode>` – The expression's AST node.
    fn parse_equality(&mut self) -> Option<ASTNode> {
        let mut expr = self.parse_comparison();

        while self.matches(&LOGIC_EQ) || self.matches(&LOGIC_NOT_EQ) {
            let opr = self.previous.clone();

            let opr_type = if let LOGIC_EQ = opr.token_type {
                BinaryExprType::LogicEQ
            } else {
                BinaryExprType::LogicNotEQ
            };

            expr = Some(Binary(BinaryExprNode {
                left: match expr {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create lhs of expression
                },
                right: match self.parse_comparison() {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create rhs of expression
                },
                opr_token: opr,
                opr_type,
            }));
        }

        return expr;
    }

    /// Parses a comparison expression as specified in the grammar.bnf file.
    ///
    /// ## Returns
    /// `Option<ASTNode>` – The expression's AST node.
    fn parse_comparison(&mut self) -> Option<ASTNode> {
        let mut expr = self.parse_range();

        while self.matches(&LESS_THAN)
            || self.matches(&LESS_THAN_EQ)
            || self.matches(&GREATER_THAN)
            || self.matches(&GREATER_THAN_EQ)
        {
            let opr = self.previous.clone();

            let opr_type = if let LESS_THAN = opr.token_type {
                BinaryExprType::LogicLessThan
            } else if let LESS_THAN_EQ = opr.token_type {
                BinaryExprType::LogicLessThanEQ
            } else if let GREATER_THAN = opr.token_type {
                BinaryExprType::LogicGreaterThan
            } else {
                BinaryExprType::LogicGreaterThanEQ
            };

            expr = Some(Binary(BinaryExprNode {
                left: match expr {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create lhs of expression
                },
                right: match self.parse_range() {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create rhs of expression
                },
                opr_token: opr,
                opr_type,
            }));
        }

        return expr;
    }

    /// Parses a range expression as specified in the grammar.bnf file.
    ///
    /// ## Returns
    /// `Option<ASTNode>` – The expression's AST node.
    fn parse_range(&mut self) -> Option<ASTNode> {
        let mut expr = self.parse_bitwise_shift();

        if self.matches(&RANGE_OPR) {
            let opr = self.previous.clone();

            expr = Some(Binary(BinaryExprNode {
                left: match expr {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create lhs of expression
                },
                right: match self.parse_bitwise_shift() {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create rhs of expression
                },
                opr_token: opr,
                opr_type: BinaryExprType::Range,
            }));
        }

        return expr;
    }

    /// Parses a 'BITWISE SHIFT' expression as specified in the grammar.bnf file.
    ///
    /// ## Returns
    /// `Option<ASTNode>` – The expression's AST node.
    fn parse_bitwise_shift(&mut self) -> Option<ASTNode> {
        let mut expr = self.parse_term();

        while self.matches(&BIT_L_SHIFT) || self.matches(&BIT_R_SHIFT) {
            let opr = self.previous.clone();

            let opr_type = if let BIT_L_SHIFT = opr.token_type {
                BinaryExprType::BitwiseShiftLeft
            } else {
                BinaryExprType::BitwiseShiftRight
            };

            expr = Some(Binary(BinaryExprNode {
                left: match expr {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create lhs of expression
                },
                right: match self.parse_term() {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create rhs of expression
                },
                opr_token: opr,
                opr_type,
            }));
        }

        return expr;
    }

    /// Parses a term expression as specified in the grammar.bnf file.
    ///
    /// ## Returns
    /// `Option<ASTNode>` – The expression's AST node.
    fn parse_term(&mut self) -> Option<ASTNode> {
        let mut expr = self.parse_factor();

        while self.matches(&PLUS) || self.matches(&MINUS) {
            let opr = self.previous.clone();

            let opr_type = if let PLUS = opr.token_type {
                BinaryExprType::Addition
            } else {
                BinaryExprType::Minus
            };

            expr = Some(Binary(BinaryExprNode {
                left: match expr {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create lhs of expression
                },
                right: match self.parse_factor() {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create rhs of expression
                },
                opr_token: opr,
                opr_type,
            }));
        }

        return expr;
    }

    /// Parses a factor expression as specified in the grammar.bnf file.
    ///
    /// ## Returns
    /// `Option<ASTNode>` – The expression's AST node.
    fn parse_factor(&mut self) -> Option<ASTNode> {
        let mut expr = self.parse_expo();

        while self.matches(&SLASH) || self.matches(&STAR) || self.matches(&MODULUS) {
            let opr = self.previous.clone();

            let opr_type = if let SLASH = opr.token_type {
                BinaryExprType::Division
            } else if let STAR = opr.token_type {
                BinaryExprType::Multiplication
            } else {
                BinaryExprType::Modulus
            };

            expr = Some(Binary(BinaryExprNode {
                left: match expr {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create lhs of expression
                },
                right: match self.parse_expo() {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create rhs of expression
                },
                opr_token: opr,
                opr_type,
            }));
        }

        return expr;
    }

    /// Parses an exponentiation expression as specified in the grammar.bnf file.
    ///
    /// ## Returns
    /// `Option<ASTNode>` – The expression's AST node.
    fn parse_expo(&mut self) -> Option<ASTNode> {
        let mut expr = self.parse_unary();

        while self.matches(&EXPO) {
            let opr = self.previous.clone();

            expr = Some(Binary(BinaryExprNode {
                left: match expr {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create lhs of expression
                },
                right: match self.parse_unary() {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create rhs of expression
                },
                opr_token: opr,
                opr_type: BinaryExprType::Expo,
            }));
        }

        return expr;
    }

    /// Parses a unary expression as specified in the grammar.bnf file.
    ///
    /// ## Returns
    /// `Option<ASTNode>` – The expression's AST node.
    fn parse_unary(&mut self) -> Option<ASTNode> {
        if self.matches(&LOGIC_NOT) || self.matches(&MINUS) || self.matches(&BIT_NOT) {
            let opr = self.previous.clone();
            let expr = self.parse_unary();

            let opr_type = if let LOGIC_NOT = opr.token_type {
                UnaryExprType::LogicNeg
            } else if let BIT_NOT = opr.token_type {
                UnaryExprType::BitwiseNeg
            } else {
                UnaryExprType::ArithmeticNeg
            };

            return Some(Unary(UnaryExprNode {
                operand: match expr {
                    Some(e) => Box::new(e),
                    None => return None, // Could not create rhs of expression
                },
                pos: (opr.line_num, opr.column_num),
                opr_type,
            }));
        } else {
            let mut expr = self.parse_primary();

            loop {
                // Parse array indexing
                if self.matches(&L_BRACKET) {
                    expr = self.parse_subscripting(expr);
                } else
                // Parse function call
                if self.matches(&L_PAREN) {
                    expr = self.parse_function_call(expr);
                } else
                // Parse object getter
                if self.matches(&DOT) {
                    let target = match expr {
                        Some(node) => Box::new(node),
                        None => return None,
                    };

                    self.consume(&&IDENTIFIER, "Expected property name after dot.");
                    let getter = self.previous.clone();

                    expr = Some(ObjectGetter(ObjectGetExprNode { target, getter }));
                } else {
                    break;
                }
            }

            return expr;
        }
    }

    /// Parses a primary (literal) expression as specified in the grammar.bnf file.
    ///
    /// ## Returns
    /// `Option<ASTNode>` – The expression's AST node.
    pub(super) fn parse_primary(&mut self) -> Option<ASTNode> {
        self.advance();

        let literal_value = match self.get_previous_tok_type() {
            STRING => self.compile_string(),
            TRUE => Object::Bool(true),
            FALSE => Object::Bool(false),
            NULL => Object::Null,
            L_BRACKET => return self.construct_array(),
            INTEGER => match self.compile_integer() {
                Ok(x) => x,
                Err(_) => return None,
            },
            FLOAT => match self.compile_float() {
                Ok(x) => x,
                Err(_) => return None,
            },
            BINARY => match self.compile_int_from_base(2) {
                Ok(x) => x,
                Err(_) => return None,
            },
            OCTAL => match self.compile_int_from_base(8) {
                Ok(x) => x,
                Err(_) => return None,
            },
            HEXADECIMAL => match self.compile_int_from_base(16) {
                Ok(x) => x,
                Err(_) => return None,
            },
            L_PAREN => {
                let start_token = self.previous.clone();

                // If the parenthesis are empty, then we parse this as an empty tuple.
                if self.matches(&R_PARENTHESIS) {
                    return Some(Tuple(TupleExprNode {
                        values: vec![],
                        token: start_token,
                    }));
                } else {
                    let expr = self.parse_expression();

                    // If there is a comma after the first expression, then this becomes a tuple.
                    return if self.matches(&COMMA) {
                        self.parse_tuple(start_token, expr)
                    } else {
                        self.consume(&R_PARENTHESIS, "Expected closing ')'.");
                        // For grouping expression, we don't wrap the inner expression inside an extra node.
                        // Instead, we return the actual expression that was enclosed in the parenthesis.
                        expr
                    };
                }
            }
            IDENTIFIER => {
                // For identifier expressions, the only information we need is enclosed within the token.
                // So we return the token wrapped inside an ASTNode::Identifier.
                return Some(Identifier(IdentifierExprNode {
                    token: self.previous.clone(),
                }));
            }
            NEW_KW => {
                // For class instances, we parse a unary after the "new" keyword so that the instance can
                // be parsed and compiled as a regular function call. The only purpose of the "new" keyword
                // is to differentiate between a function call and a class instance in Hinton code.
                let instance = match self.parse_unary() {
                    Some(i) => i,
                    None => return None, // could not parse instance
                };

                return match instance {
                    ASTNode::FunctionCall(call) => Some(Instance(call)),
                    _ => {
                        self.error_at_current("Expected class instance.");
                        None
                    }
                };
            }
            _ => {
                self.error_at_previous("Unexpected token.");
                return None;
            }
        };

        let node = LiteralExprNode {
            value: literal_value,
            token: self.current.clone(),
        };

        return Some(Literal(node));
    }

    /// Compiles a string token to a Hinton String.
    ///
    /// ## Returns
    /// `Rc<Object>` – The Hinton string object.
    fn compile_string(&mut self) -> Object {
        let lexeme = self.previous.lexeme.clone();

        // Remove outer quotes from the source string
        let lexeme = &lexeme[1..(lexeme.len() - 1)];

        // Replace escaped characters with the actual representations
        let lexeme = lexeme
            .replace("\\n", "\n")
            .replace("\\t", "\t")
            .replace("\\r", "\r")
            .replace("\\\\", "\\")
            .replace("\\\"", "\"");

        // Emits the constant instruction
        return Object::String(lexeme);
    }

    /// Compiles an integer token to a Hinton Int.
    ///
    /// ## Returns
    /// `Rc<Object>` – The Hinton number object.
    fn compile_integer(&mut self) -> Result<Object, ()> {
        let lexeme = self.previous.lexeme.clone();
        // Removes the underscores from the lexeme
        let lexeme = lexeme.replace('_', "");
        // Parses the lexeme into a float
        let num = lexeme.parse::<i64>();

        // If the lexeme could successfully be converted to `isize` integer
        // then we proceed to save it in the constant pool and emit the
        // instruction. Otherwise, we indicate that there was a compilation error.
        return match num {
            Ok(x) => Ok(Object::Int(x)),
            Err(_) => {
                // This should almost never happen.
                self.error_at_previous("Unexpected token.");
                Err(())
            }
        };
    }

    /// Compiles a float token to a Hinton Float.
    ///
    /// ## Returns
    /// `Rc<Object>` – The Hinton number object.
    fn compile_float(&mut self) -> Result<Object, ()> {
        let lexeme = self.previous.lexeme.clone();
        // Removes the underscores from the lexeme
        let lexeme = lexeme.replace('_', "");
        // Parses the lexeme into a float
        let num = lexeme.parse::<f64>();

        // If the lexeme could successfully be converted to `isize` integer
        // then we proceed to save it in the constant pool and emit the
        // instruction. Otherwise, we indicate that there was a compilation error.
        return match num {
            Ok(x) => Ok(Object::Float(x)),
            Err(_) => {
                // This should almost never happen.
                self.error_at_previous("Unexpected token.");
                Err(())
            }
        };
    }

    /// Compiles a binary, octal, or hexadecimal number token to a Hinton Number.
    ///
    /// ## Returns
    /// `Result<Object, ()>` – If there was no error converting the lexeme to an integer
    /// of the specified base, returns the Hinton number object. Otherwise, returns an empty error.
    fn compile_int_from_base(&mut self, radix: u32) -> Result<Object, ()> {
        let lexeme = self.previous.lexeme.clone();
        // Removes the underscores from the lexeme
        let lexeme = lexeme.replace('_', "");
        // Parses the lexeme into an integer
        let num = isize::from_str_radix(&lexeme[2..], radix);

        // If the lexeme could successfully be converted to `isize` integer
        // then we proceed to save it in the constant pool and emit the
        // instruction. Otherwise, we indicate that there was a compilation error.
        return match num {
            Ok(x) => Ok(Object::Int(x as i64)),
            Err(_) => {
                // This should almost never happen.
                self.error_at_previous("Unexpected token.");
                Err(())
            }
        };
    }

    /// Parses an array expression as specified in the grammar.bnf file.
    ///
    /// ## Returns
    /// `Option<ASTNode>` – The expression's AST node.
    fn construct_array(&mut self) -> Option<ASTNode> {
        let start_token = self.previous.clone();
        let mut values: Vec<Box<ASTNode>> = vec![];

        if !self.matches(&R_BRACKET) {
            loop {
                values.push(match self.parse_expression() {
                    Some(e) => Box::new(e),
                    None => return None,
                });

                if self.matches(&COMMA) {
                    continue;
                }

                self.consume(&R_BRACKET, "Expected ']' after array declaration.");

                break;
            }
        }

        return Some(Array(ArrayExprNode {
            values,
            token: start_token,
        }));
    }

    /// Parses a tuple expression as specified in the grammar.bnf file.
    ///
    /// ## Returns
    /// `Option<ASTNode>` – The expression's AST node.
    fn parse_tuple(&mut self, start_token: Token, first: Option<ASTNode>) -> Option<ASTNode> {
        let first = match first {
            Some(node) => Box::new(node),
            None => return None, // The first expression is invalid.
        };

        // Initialize the vector
        let mut values: Vec<Box<ASTNode>> = vec![first];

        if !self.matches(&R_PARENTHESIS) {
            loop {
                values.push(match self.parse_expression() {
                    Some(e) => Box::new(e),
                    None => return None,
                });

                if self.matches(&COMMA) {
                    continue;
                }

                self.consume(&R_PARENTHESIS, "Expected ')' after tuple declaration.");
                break;
            }
        }

        return Some(Tuple(TupleExprNode {
            values,
            token: start_token,
        }));
    }

    /// Parses an array indexing expression as specified in the grammar.bnf file.
    ///
    /// ## Returns
    /// `Option<ASTNode>` – The expression's AST node.
    fn parse_subscripting(&mut self, expr: Option<ASTNode>) -> Option<ASTNode> {
        let expr = match expr {
            Some(e) => e,
            None => return None,
        };

        let pos = (self.previous.line_num, self.previous.column_num);

        let expr = Some(ArrayIndexing(ArrayIndexingExprNode {
            target: Box::new(expr),
            index: match self.parse_expression() {
                Some(e) => Box::new(e),
                None => return None,
            },
            pos,
        }));

        self.consume(&R_BRACKET, "Expected ']' after array index.");
        return expr;
    }

    /// Parses an function call expression as specified in the grammar.bnf file.
    ///
    /// ## Returns
    /// `Option<ASTNode>` – The expression's AST node.
    fn parse_function_call(&mut self, name: Option<ASTNode>) -> Option<ASTNode> {
        let name = match name {
            Some(e) => e,
            None => return None,
        };

        let pos = (self.previous.line_num, self.previous.column_num);
        let mut args: Vec<Argument> = vec![];

        if !self.matches(&R_PARENTHESIS) {
            loop {
                if args.len() >= 255 {
                    self.error_at_current("Can't have more than 255 arguments.");
                    return None;
                }

                match self.parse_argument() {
                    Some(a) => {
                        if args.len() > 0 && !a.is_named && args.last().unwrap().is_named {
                            self.error_at_previous(
                                "Named arguments must be declared after all unnamed arguments.",
                            );
                            return None;
                        }

                        args.push(a);
                    }
                    None => return None, // Could not parse the argument
                }

                if self.matches(&COMMA) {
                    continue;
                }

                self.consume(&&R_PARENTHESIS, "Expected ')' after arguments.");
                break;
            }
        }

        Some(FunctionCall(FunctionCallExprNode {
            target: Box::new(name),
            args,
            pos,
        }))
    }

    /// Parses a function argument expression as specified in the grammar.bnf file.
    ///
    /// ## Returns
    /// `Option<ASTNode>` – The expression's AST node.
    fn parse_argument(&mut self) -> Option<Argument> {
        let expr = match self.parse_expression() {
            Some(e) => e,
            None => return None, // could not parse argument expression
        };

        if self.matches(&COLON_EQUALS) {
            return Some(Argument {
                name: Some(expr),
                is_named: true,
                value: match self.parse_expression() {
                    Some(x) => Box::new(x),
                    None => return None, // Could not compile default value for parameter
                },
            });
        }

        Some(Argument {
            name: None,
            is_named: false,
            value: Box::new(expr),
        })
    }
}
