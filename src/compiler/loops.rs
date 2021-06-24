use crate::ast::*;
use crate::bytecode::OpCode;
use crate::compiler::symbols::SymbolType;
use crate::compiler::{BreakScope, Compiler, LoopScope, LoopType};
use crate::errors::CompilerErrorType;
use crate::lexer::tokens::Token;

impl Compiler {
   /// Compiles a `while` statement.
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
         scope_depth: depth,
      });

      // Only compile the condition if it is not a truthy literal or equivalent.
      let mut exit_jump = 0;
      if !condition_is_truthy_lit {
         self.compile_node(&stmt.condition);
         exit_jump = self.emit_jump(OpCode::PopJumpIfFalse, &stmt.token);
      }

      self.compile_node(&stmt.body);

      // Jump to the start of the loop (including the re-execution of the condition)
      self.emit_loop(loop_start, &stmt.token);

      // If the condition is not a truthy literal, then we must patch the 'OP_JUMP_IF_FALSE' above
      if !condition_is_truthy_lit {
         self.patch_jump(exit_jump, &stmt.token);
      }

      self.close_breaks(loop_start, &stmt.token);
      self.current_func_scope_mut().loops.pop(); // ends this loop's break scope
   }

   /// Compiles a `for` statement.
   pub(super) fn compile_for_stmt(&mut self, stmt: &ForStmtNode) {
      let loop_line_info = (stmt.token.line_num, stmt.token.column_start);

      // Create the iterator at runtime
      self.compile_node(&stmt.iterator);
      self.emit_op_code(OpCode::MakeIter, loop_line_info);

      // Begin the loop
      let loop_start = self.current_chunk().len();
      let exit_jump = self.emit_jump(OpCode::ForIterNextOrJump, &stmt.token);

      // Increment the scope for the loop's iterator
      self.current_func_scope_mut().scope_depth += 1;

      // Emits a placeholder symbol for the loop's iterator, which lives on the stack
      // until the end of the loop. The programmer will never be able to access this
      // symbol's value directly because of the format of its name.
      match self.emit_symbol(
         &format!("<for-loop at #{}>", loop_start),
         &stmt.token,
         SymbolType::Constant,
      ) {
         Ok(symbol_pos) => self.current_func_scope_mut().s_table.mark_initialized(symbol_pos),
         Err(_) => return,
      }

      // Increment the scope for the loop's body.
      // The call to `self.end_scope(...)` below removes this scope.
      self.current_func_scope_mut().scope_depth += 1;

      // Starts this loop's break scope
      let depth = self.relative_scope_depth();
      self.current_func_scope_mut().loops.push(LoopScope {
         position: loop_start,
         loop_type: LoopType::ForIn,
         scope_depth: depth,
      });

      // Declares the loop's identifier.
      match self.declare_symbol(&stmt.id.token, SymbolType::Variable) {
         Ok(symbol_pos) => self.current_func_scope_mut().s_table.mark_initialized(symbol_pos),
         Err(_) => return,
      }

      // Compiles the loop's body
      for node in stmt.body.iter() {
         self.compile_node(node);
      }

      // Ends the scope for the loop's body.
      self.end_scope(&stmt.token);

      // Jump to the start of the loop
      self.emit_loop(loop_start, &stmt.token);
      self.patch_jump(exit_jump, &stmt.token);

      // Closes & patches all breaks associated with this loop
      self.close_breaks(loop_start, &stmt.token);

      // Ends this loop's break scope
      self.current_func_scope_mut().loops.pop();

      // Removes the loop's iterator and ends the iterator scope.
      self.current_func_scope_mut().s_table.pop();
      self.current_func_scope_mut().scope_depth -= 1;
   }

   /// Compiles a `break` statement.
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
         self
            .current_func_scope_mut()
            .s_table
            .pop_scope(current_loop.scope_depth, false, false);

      // If we are branching inside a for-in loop, also pop the loop's
      // iterator off the stack before exiting the loop.
      if let super::LoopType::ForIn = current_loop.loop_type {
         if stmt.is_break {
            popped_scope.append(&mut vec![false]);
         }
      }

      // Emit the pop instructions
      self.emit_stack_pops(popped_scope, &stmt.token);

      if stmt.is_break {
         // Jump out of the loop
         let break_pos = self.emit_jump(OpCode::JumpForward, &stmt.token);

         // Adds this break to the breaks list associated with the current loop so that it can
         // be patched later.
         self.current_func_scope_mut().breaks.push(BreakScope {
            parent_loop: current_loop,
            chunk_pos: break_pos,
         })
      } else {
         self.emit_loop(current_loop.position, &stmt.token);
      }
   }

   /// Patches all breaks associated with the current loop.
   ///
   /// # Parameters
   /// - `loop_start`: The position in the chunk of the loop instruction associated with this
   /// break statement.
   /// - `token`: A reference to the token associated with this break statement's loop
   fn close_breaks(&mut self, loop_start: usize, token: &Token) {
      // Looks for any break statements associated with this loop
      let mut breaks: Vec<usize> = vec![];
      for b in self
         .current_function_scope()
         .breaks
         .iter()
         .filter(|br| br.parent_loop.position == loop_start)
      {
         breaks.push(b.chunk_pos);
      }

      // Patches all the breaks associated with this loop
      for b in breaks {
         self.patch_jump(b, token);
      }
   }
}
