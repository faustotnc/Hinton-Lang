use super::{
    symbols::{Symbol, SymbolType},
    Compiler,
};
use crate::{ast::*, bytecode::OpCode, errors::CompilerErrorType, lexer::tokens::Token, objects::Object};
use std::borrow::Borrow;

impl Compiler {
    /// Compiles an expression statement.
    ///
    /// * `stmt` – The expression statement node being compiled.
    pub(super) fn compile_expression_stmt(&mut self, stmt: &ExpressionStmtNode) {
        self.compile_node(&stmt.child);
        self.emit_op_code(OpCode::PopStackTop, stmt.pos);
    }

    /// Compiles a variable declaration.
    ///
    /// ## Arguments
    /// * `expr` – A variable declaration node.
    pub(super) fn compile_variable_decl(&mut self, decl: &VariableDeclNode) {
        // Declares the variables
        for id in decl.identifiers.iter() {
            if let Ok(symbol_pos) = self.declare_symbol(id, SymbolType::Variable) {
                // Compiles the variable's value
                self.compile_node(&decl.value);

                // If we are in the global scope, declarations are
                // stored in the VM.globals hashmap
                if self.is_global_scope() {
                    self.define_as_global(id);
                    self.globals.mark_initialized(symbol_pos);
                } else {
                    // Marks the variables as initialized, a.k.a, defines the variables.
                    self.current_func_scope_mut().s_table.mark_initialized(symbol_pos);
                }
            }
        }
    }

    /// Compiles a constant declaration.
    ///
    /// ## Arguments
    /// * `expr` – A constant declaration node.
    pub(super) fn compile_constant_decl(&mut self, decl: &ConstantDeclNode) {
        // Declares the constant
        if let Ok(symbol_pos) = self.declare_symbol(&decl.name, SymbolType::Constant) {
            // Compiles the constant's value
            self.compile_node(&decl.value);

            // If we are in the global scope, declarations are
            // stored in the VM.globals hashmap
            if self.is_global_scope() {
                self.define_as_global(&decl.name);
                self.globals.mark_initialized(symbol_pos);
            } else {
                self.current_func_scope_mut().s_table.mark_initialized(symbol_pos);
            }
        }
    }

    /// Defines a declaration as global by emitting a `DEFINE_GLOBAL` instructions.
    ///
    /// ## Arguments
    /// * `token` – The token associated with this global declaration.
    pub(super) fn define_as_global(&mut self, token: &Token) {
        let name = Object::String(token.lexeme.clone());
        if let Some(idx) = self.add_literal_to_pool(name, token, false) {
            let pos = (token.line_num, token.column_num);

            if idx < 256 {
                self.emit_op_code_with_byte(OpCode::DefineGlobal, idx as u8, pos);
            } else {
                self.emit_op_code_with_short(OpCode::DefineGlobalLong, idx, pos);
            }
        }
    }

    /// Declares the symbol by adding it to the symbol table
    ///
    /// ## Arguments
    /// * `token` – The token (symbol name) related to the symbol being declared.
    /// * `symbol_type` – The type of symbol being declared.
    ///
    /// ## Returns
    /// `Result<(), ()>` – Whether or not there was an error with the variable declaration.
    pub(super) fn declare_symbol(&mut self, token: &Token, symbol_type: SymbolType) -> Result<usize, ()> {
        let depth = self.relative_scope_depth();

        if let Some(symbol) = self
            .current_func_scope_mut()
            .s_table
            .find_in_scope(&token.lexeme, depth)
        {
            match symbol.symbol_type {
                SymbolType::Variable
                | SymbolType::Constant
                | SymbolType::Function
                | SymbolType::Class
                | SymbolType::Enum => self.error_at_token(
                    &token,
                    CompilerErrorType::Duplication,
                    &format!("Duplicate definition for identifier '{}'", token.lexeme),
                ),
                SymbolType::Parameter => self.error_at_token(
                    &token,
                    CompilerErrorType::Duplication,
                    &format!("Duplicate definition for parameter '{}'", token.lexeme),
                ),
            }

            return Err(());
        }

        self.emit_symbol(&token, symbol_type)
    }

    /// Tries to emit a symbol declaration into the symbol table.
    ///
    /// ## Arguments
    /// * `token` – The token (symbol name) related to the symbol being emitted.
    /// * `symbol_type` – The type of symbol being emitted.
    ///
    /// ## Returns
    /// `Result<(), ()>` – Whether or not there was an error with the symbol declaration.
    fn emit_symbol(&mut self, name: &Token, symbol_type: SymbolType) -> Result<usize, ()> {
        self.push_symbol(
            name,
            Symbol {
                name: name.lexeme.clone(),
                symbol_depth: self.relative_scope_depth(),
                is_initialized: match symbol_type {
                    SymbolType::Variable | SymbolType::Constant | SymbolType::Function => false,
                    _ => true,
                },
                symbol_type,
                is_used: false,
                line_info: (name.line_num, name.column_num),
                is_captured: false,
            },
        )
    }

    pub(super) fn push_symbol(&mut self, token: &Token, symbol: Symbol) -> Result<usize, ()> {
        if self.is_global_scope() {
            self.globals.push(symbol);
            Ok(self.globals.len() - 1)
        } else {
            if self.current_func_scope_mut().s_table.len() >= (u16::MAX as usize) {
                self.error_at_token(
                    &token,
                    CompilerErrorType::MaxCapacity,
                    "Too many variables in this scope.",
                );
                return Err(());
            }

            self.current_func_scope_mut().s_table.push(symbol);
            // Variable was successfully declared
            Ok(self.current_func_scope_mut().s_table.len() - 1)
        }
    }

    /// Compiles a block statement.
    ///
    /// * `block` – The block node being compiled.
    pub(super) fn compile_block_stmt(&mut self, block: &BlockNode) {
        if !block.is_func_body {
            self.begin_scope();
        }

        for node in block.body.iter() {
            self.compile_node(&node.clone());
        }

        self.end_scope(block.is_func_body, &block.end_of_block);
    }

    /// Starts a new scope.
    fn begin_scope(&mut self) {
        self.current_func_scope_mut().scope_depth += 1;
    }

    /// Ends a scope.
    fn end_scope(&mut self, is_func_body: bool, token: &Token) {
        let scope = self.relative_scope_depth();

        let popped_scope = self
            .current_func_scope_mut()
            .s_table
            .pop_scope(scope, !is_func_body, true);

        if !is_func_body {
            self.emit_stack_pops(popped_scope, token);
            self.current_func_scope_mut().scope_depth -= 1;
        }
    }

    pub(super) fn emit_stack_pops(&mut self, symbols: Vec<bool>, token: &Token) {
        let pos = (token.line_num, token.column_num);

        for is_closed in symbols {
            self.emit_op_code(
                if is_closed {
                    OpCode::PopCloseUpVal
                } else {
                    OpCode::PopStackTop
                },
                pos,
            );
        }
    }

    /// Compiles an if statement.
    ///
    /// * `block` – The if statement node being compiled.
    pub(super) fn compile_if_stmt(&mut self, stmt: &IfStmtNode) {
        let condition_is_lit_true = stmt.condition.is_truthy_literal();
        let condition_is_lit_false = stmt.condition.is_false_literal();

        let mut then_jump = 0;
        // Only execute the condition if it is not a boolean literal (or equivalent).
        if !condition_is_lit_true && !condition_is_lit_false {
            // Compiles the condition so that its value is at the top of the
            // stack during runtime. This value is then checked for truthiness
            // to execute the correct branch of the if statement.
            self.compile_node(&stmt.condition);
            then_jump = self.emit_jump(OpCode::PopJumpIfFalse, &stmt.then_token);
        }

        // If the condition is always false, the `then`
        // branch does not need to be compiled at all.
        if !condition_is_lit_false {
            self.compile_node(&stmt.then_branch);
        }

        // If the condition is always true, then the `else`
        // branch does not need to be compiled at all.
        if condition_is_lit_true {
            return;
        }

        // If the condition is always false, then we only care about
        // compiling the else branch. However, if there is no else branch
        // we return out of the function because there is nothing else to do.
        if condition_is_lit_false {
            match stmt.else_branch.borrow() {
                Some(else_branch) => self.compile_node(&else_branch),
                None => return,
            }

            return;
        }

        let else_jump = match stmt.else_token.borrow() {
            Some(token) => self.emit_jump(OpCode::JumpForward, &token),
            // We are okay to return a dummy value because the only way `else_jump` can
            // be used is if there was an `else` branch in the first place. If there is
            // no `else` token, then there is no `else` branch, which means that the bellow
            // match statement will not execute, and so this value will not be used.
            None => 0,
        };

        if !condition_is_lit_false {
            self.patch_jump(then_jump, &stmt.then_token);
        }

        if let Some(else_branch) = stmt.else_branch.borrow() {
            self.compile_node(&else_branch);
            // Because at this point we *do* have an 'else' branch, we know that for sure
            // these is an `else_token`, so it is safe to unwrap without check.
            self.patch_jump(else_jump, &stmt.else_token.clone().unwrap());
        }
    }
}
