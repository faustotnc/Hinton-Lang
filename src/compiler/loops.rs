use super::{BreakScope, Compiler, LoopScope};
use crate::{
    ast::{ForStmtNode, LoopBranchStmtNode, WhileStmtNode},
    bytecode::OpCode,
    compiler::{CompilerErrorType, LoopType, Symbol, SymbolType},
    lexer::tokens::Token,
};

impl Compiler {
    /// Compiles a `while` statement.
    ///
    /// * `stmt` – The `while` statement node being compiled.
    pub(super) fn compile_while_stmt(&mut self, stmt: &WhileStmtNode) {
        // We don't need to compile the loop if the condition is a
        // `false` literal because it will never execute.
        if stmt.condition.is_false_literal() {
            return;
        }

        let condition_is_truthy_lit = stmt.condition.is_truthy_literal();

        let loop_start = self.current_chunk().len();
        // starts this loop's break scope
        let depth = self.relative_scope_depth() + 1;
        self.current_func_scope_mut().loops.push(LoopScope {
            position: loop_start,
            loop_type: LoopType::While,
            // +1 because we don't start the actual scope until the loop
            // body is being compiled, which occurs later in this function.
            scope_depth: depth,
        });

        // Only compile the condition if it is not a truthy literal or equivalent.
        let mut exit_jump = 0;
        if !condition_is_truthy_lit {
            self.compile_node(&stmt.condition);
            exit_jump = self.emit_jump(OpCode::PopJumpIfFalse, &stmt.token);
        }

        self.compile_node(&stmt.body);

        // Jump back to the start of the loop (including the re-execution of the condition)
        self.emit_loop(loop_start, &stmt.token);

        // If the condition is not a truthy literal, then we must patch the 'OP_JUMP_IF_FALSE' above
        if !condition_is_truthy_lit {
            self.patch_jump(exit_jump, &stmt.token);
        }

        self.close_breaks(loop_start, &stmt.token);
        self.current_func_scope_mut().loops.pop(); // ends this loop's break scope
    }

    /// Compiles a `for` statement.
    ///
    /// * `stmt` – The `for` statement node being compiled.
    pub(super) fn compile_for_stmt(&mut self, stmt: &ForStmtNode) {
        let loop_line_info = (stmt.token.line_num, stmt.token.column_num);

        // Create the iterator at runtime
        self.compile_node(&stmt.iterator);
        self.emit_op_code(OpCode::MakeIter, loop_line_info);

        // Increment the scope so that the loop's identifier and iterator
        // placeholder have their own scope.
        self.current_func_scope_mut().scope_depth += 1;

        // Begin the loop
        let loop_start = self.current_chunk().len();
        let exit_jump = self.emit_jump(OpCode::ForIterNextOrJump, &stmt.token);

        // Starts this loop's break scope
        let depth = self.relative_scope_depth() + 1;
        self.current_func_scope_mut().loops.push(LoopScope {
            position: loop_start,
            loop_type: LoopType::ForIn,
            // +1 because we don't start the actual scope until the loop
            // body is being compiled, which occurs later in this function.
            scope_depth: depth,
        });

        // Emits a placeholder symbol for the loop's iterator, which lives on
        // the stack until the end of the loop. The programmer will never be
        // able to access this "variable" because of the format of its name.
        match self.push_symbol(
            &stmt.token,
            Symbol {
                name: format!("<for-loop placeholder at byte #{}>", loop_start),
                symbol_depth: self.relative_scope_depth(),
                is_initialized: true,
                symbol_type: SymbolType::Constant,
                is_used: true,
                line_info: loop_line_info,
                is_captured: false,
            },
        ) {
            Ok(symbol_pos) => self.current_func_scope_mut().s_table.mark_initialized(symbol_pos),
            Err(_) => return,
        }

        // Declares the loop's identifier.
        match self.declare_symbol(&stmt.id.token, SymbolType::Variable) {
            Ok(symbol_pos) => self.current_func_scope_mut().s_table.mark_initialized(symbol_pos),
            Err(_) => return,
        }

        // Compiles the loop's body
        self.compile_node(&stmt.body);

        // Either pops the loop variable off the stack or lifts it
        // as a closed UpValRef if it is captured by a function
        // declared inside the loop.
        if self
            .current_function_scope()
            .s_table
            .symbols
            .last()
            .unwrap()
            .is_captured
        {
            self.emit_op_code(OpCode::PopCloseUpVal, loop_line_info);
        } else {
            self.emit_op_code(OpCode::PopStackTop, loop_line_info);
        }

        // Jump back to the start of the loop (including the re-execution of the condition)
        self.emit_loop(loop_start, &stmt.token);
        self.patch_jump(exit_jump, &stmt.token);

        // Closes & patches all breaks associated with this loop
        self.close_breaks(loop_start, &stmt.token);

        // Ends this loop's break scope
        self.current_func_scope_mut().loops.pop();

        // Remove the loop's variables and end the scope
        self.current_func_scope_mut().s_table.pop();
        self.current_func_scope_mut().s_table.pop();
        self.current_func_scope_mut().scope_depth -= 1;
    }

    /// Patches all breaks associated with the current loop.
    fn close_breaks(&mut self, loop_start: usize, token: &Token) {
        // Looks for any break statements associated with this loop
        let mut breaks: Vec<usize> = vec![];
        for b in self
            .current_func_scope_mut()
            .breaks
            .iter()
            .filter(|br| br.parent_loop.position == loop_start)
        {
            breaks.push(b.loop_position);
        }

        // Patches all the breaks associated with this loop
        for b in breaks {
            self.patch_jump(b, token);
        }
    }

    /// Compiles a `break` statement.
    ///
    /// * `stmt` – The `break` statement node being compiled.
    pub(super) fn compile_loop_branching_stmt(&mut self, stmt: &LoopBranchStmtNode) {
        if self.current_function_scope().loops.len() == 0 {
            self.error_at_token(
                &stmt.token,
                CompilerErrorType::Syntax,
                &format!(
                    "Cannot have '{}' statement outside of loop.",
                    if stmt.is_break { "break" } else { "continue" }
                ),
            );
            return;
        }

        let current_loop = *self.current_function_scope().loops.last().unwrap();
        let mut popped_scope =
            self.current_func_scope_mut()
                .s_table
                .pop_scope(current_loop.scope_depth, false, false);

        // If we are branching inside a for-in loop, also pop the loop's variable
        // and the iterator off the stack before exiting the loop.
        if let super::LoopType::ForIn = current_loop.loop_type {
            if stmt.is_break {
                popped_scope.append(&mut vec![false, false]);
            } else {
                popped_scope.append(&mut vec![false]);
            }
        }

        // Emit the pop instructions
        self.emit_stack_pops(popped_scope, &stmt.token);

        if stmt.is_break {
            // Jump out of the loop
            let break_pos = self.emit_jump(OpCode::JumpForward, &stmt.token);

            // Adds this break to the breaks list associated with the current loop
            // so that it can be patched later.
            self.current_func_scope_mut().breaks.push(BreakScope {
                parent_loop: current_loop,
                loop_position: break_pos,
            })
        } else {
            self.emit_loop(current_loop.position, &stmt.token);
        }
    }
}
