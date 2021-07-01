use crate::ast::*;
use crate::bytecode::OpCode;
use crate::compiler::symbols::{Symbol, SymbolTable, SymbolType};
use crate::compiler::{ClassScope, Compiler, CompilerCtx};
use crate::errors::CompilerErrorType;
use crate::lexer::tokens::Token;
use crate::objects::Object;
use std::borrow::Borrow;

impl Compiler {
   /// Compiles an expression statement.
   pub(super) fn compile_expression_stmt(&mut self, stmt: &ExpressionStmtNode) {
      self.compile_node(&stmt.child);
      self.emit_op_code(OpCode::PopStackTop, stmt.pos);
   }

   /// Compiles a variable declaration.
   pub(super) fn compile_variable_decl(&mut self, decl: &VariableDeclNode) {
      // Get the symbol type for the function declaration.
      let s_type = if let CompilerCtx::Class = self.compiler_type {
         SymbolType::VarField
      } else {
         SymbolType::Var
      };

      for id in decl.identifiers.iter() {
         if let Ok(symbol_pos) = self.declare_symbol(id, s_type.clone()) {
            self.compile_node(&decl.value);

            // If the compiler is currently compiling a class, append the variable to the class.
            if let CompilerCtx::Class = self.compiler_type {
               if self
                  .add_literal_to_pool(Object::String(id.lexeme.clone()), id, true)
                  .is_some()
               {
                  self.emit_op_code(OpCode::AppendVarField, (id.line_num, id.column_start));
               }
            }

            if self.is_global_scope() {
               self.define_as_global(id);
            }

            self.current_s_table_mut().mark_initialized(symbol_pos)
         }
      }
   }

   /// Compiles a constant declaration.
   pub(super) fn compile_constant_decl(&mut self, decl: &ConstantDeclNode) {
      // Get the symbol type for the function declaration.
      let s_type = if let CompilerCtx::Class = self.compiler_type {
         SymbolType::ConstField
      } else {
         SymbolType::Const
      };

      if self.declare_symbol(&decl.name, s_type).is_ok() {
         self.compile_node(&decl.value);

         // If the compiler is currently compiling a class, append the variable to the class.
         if let CompilerCtx::Class = self.compiler_type {
            if self
               .add_literal_to_pool(Object::String(decl.name.lexeme.clone()), &decl.name, true)
               .is_some()
            {
               self.emit_op_code(
                  OpCode::AppendConstField,
                  (decl.name.line_num, decl.name.column_start),
               );
            }
         }

         if self.is_global_scope() {
            self.define_as_global(&decl.name);
         }
      }
   }

   /// Defines a declaration as global by emitting a `DEFINE_GLOBAL` instructions.
   pub(super) fn define_as_global(&mut self, token: &Token) {
      if let Some(idx) = self.add_literal_to_pool(Object::String(token.lexeme.clone()), token, false) {
         let pos = (token.line_num, token.column_start);

         if idx < 256 {
            self.emit_op_code_with_byte(OpCode::DefineGlobal, idx as u8, pos);
         } else {
            self.emit_op_code_with_short(OpCode::DefineGlobalLong, idx, pos);
         }
      }
   }

   /// Declares the symbol by adding it to the symbol table.
   ///
   /// # Parameters
   /// - `token`: The token (symbol name) related to the symbol to be declared.
   /// - `symbol_type`: The type of symbol to be declared.
   ///
   /// # Returns
   /// `Result<usize, ()>`: If the declaration was okay, returns the position of the
   /// position of the symbol in the symbol table.
   pub(super) fn declare_symbol(&mut self, token: &Token, symbol_type: SymbolType) -> Result<usize, ()> {
      let depth = self.relative_scope_depth();

      if let Some(symbol) = self.current_s_table_mut().lookup(&token.lexeme, depth) {
         let msg = match symbol.s_type {
            SymbolType::Param => "parameter",
            SymbolType::Method | SymbolType::VarField | SymbolType::ConstField => "class member",
            _ => "identifier",
         };

         self.error_at_token(
            &token,
            CompilerErrorType::Duplication,
            &format!("Duplicate definition for {} '{}'.", msg, token.lexeme),
         );

         return Err(());
      }

      self.emit_symbol(&token.lexeme, &token, symbol_type)
   }

   /// Tries to emit a symbol declaration into the symbol table of the current function
   /// or into the global symbol table for the compiler.
   ///
   /// # Parameters
   /// - `name`: The symbol's name.
   /// - `token`: The token related to the symbol to be emitted.
   /// - `symbol_type`: The type of symbol to be emitted.
   ///
   /// # Returns
   /// `Result<usize, ()>`: If the declaration was okay, returns the position of the symbol in the symbol table.
   pub(super) fn emit_symbol(&mut self, name: &str, token: &Token, st: SymbolType) -> Result<usize, ()> {
      let symbol = Symbol {
         name: name.to_string(),
         depth: self.relative_scope_depth(),
         is_initialized: !matches!(
            st,
            SymbolType::Var
               | SymbolType::Const
               | SymbolType::Func
               | SymbolType::ConstField
               | SymbolType::Method
               | SymbolType::VarField
         ),
         s_type: st,
         is_used: false,
         line_info: (token.line_num, token.column_start),
         is_captured: false,
      };

      if self.is_global_scope() {
         self.globals.push(symbol);
         Ok(self.globals.len() - 1)
      } else {
         if self.current_s_table().len() >= (u16::MAX as usize) {
            self.error_at_token(
               &token,
               CompilerErrorType::MaxCapacity,
               "Too many local variables in this block.",
            );
            return Err(());
         }

         self.current_s_table_mut().push(symbol);
         Ok(self.current_s_table().len() - 1)
      }
   }

   /// Compiles a block statement.
   pub(super) fn compile_block_stmt(&mut self, block: &BlockNode) {
      self.current_func_scope_mut().scope_depth += 1;

      for node in block.body.iter() {
         self.compile_node(node);
      }

      self.end_scope(&block.end_of_block);
   }

   /// Ends the current scope (and removes the symbols in the popped scope).
   ///
   /// # Parameters
   /// - `token`: The token associated with the end of the scope.
   pub(super) fn end_scope(&mut self, token: &Token) {
      let scope = self.relative_scope_depth();
      let popped_scope = self.current_func_scope_mut().s_table.pop_scope(scope, true, true);

      self.emit_stack_pops(popped_scope, token);
      self.current_func_scope_mut().scope_depth -= 1;
   }

   /// Emits either a `PopStackTop`, or a `PopCloseUpVal` instruction for each symbol in the
   /// provided popped-symbols vector.
   ///
   /// # Parameters
   /// * `symbols`: A vector of popped symbols.
   /// * `token`: The token associated with the pop instructions.
   pub(super) fn emit_stack_pops(&mut self, symbols: Vec<bool>, token: &Token) {
      let pos = (token.line_num, token.column_start);

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
         // there is an `else_token`, so it is safe to unwrap without check.
         self.patch_jump(else_jump, &stmt.else_token.clone().unwrap());
      }
   }

   /// Compiles a class declaration statement.
   pub(super) fn compile_class_declaration(&mut self, decl: &ClassDeclNode) {
      if self.declare_symbol(&decl.name, SymbolType::Class).is_ok() {
         let str_name = Object::String(decl.name.lexeme.clone());
         let name_line_info = (decl.name.line_num, decl.name.column_start);

         // Adds this class to the list of class scopes
         self.classes.push(ClassScope {
            members: SymbolTable::new(vec![]),
         });

         // Adds the class's name to the pool
         let name_pool_pos = match self.add_literal_to_pool(str_name, &decl.name, false) {
            Some(p) => p,
            None => return,
         };

         // Changes the compiler's context to a class
         let prev_compiler_type = std::mem::replace(&mut self.compiler_type, CompilerCtx::Class);

         // Make the class object at runtime.
         if name_pool_pos < 256 {
            self.emit_op_code_with_byte(OpCode::MakeClass, name_pool_pos as u8, name_line_info)
         } else {
            self.emit_op_code_with_short(OpCode::MakeClass, name_pool_pos, name_line_info)
         }

         // Emits the class methods
         for method in decl.members.iter() {
            match &method.member_type {
               ClassMemberDecl::Var(v) => self.compile_variable_decl(v),
               ClassMemberDecl::Const(c) => self.compile_constant_decl(c),
               ClassMemberDecl::Method(m) => {
                  if m.name.lexeme == "init" {
                     self.compile_function_decl(m, CompilerCtx::Init)
                  } else {
                     self.compile_function_decl(m, CompilerCtx::Method)
                  }
               }
            }
         }

         // Return the compiler to its previous context
         self.compiler_type = prev_compiler_type;

         // Define the class as a global symbol if we are in the global scope.
         if self.is_global_scope() {
            self.define_as_global(&decl.name);
         }

         // Removes this class from the list of class scopes.
         self.classes.pop();
      }
   }
}
