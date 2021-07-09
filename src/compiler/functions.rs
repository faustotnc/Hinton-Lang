use crate::compiler::symbols::{Symbol, SymbolTable, SymbolType};
use crate::compiler::{Compiler, CompilerCtx, FunctionScope, UpValue};
use crate::core::ast::*;
use crate::core::bytecode::OpCode;
use crate::core::chunk::Chunk;
use crate::core::tokens::Token;
use crate::errors::CompilerErrorType;
use crate::objects::{FuncObject, Object};

impl Compiler {
   /// Compiles a function declaration statement.
   pub(super) fn compile_function_decl(&mut self, decl: &FunctionDeclNode, t: CompilerCtx) {
      let func_pos = (decl.name.line_num, decl.name.column_start);

      // Get the symbol type for the function declaration.
      let s_type = if matches!(t, CompilerCtx::Method | CompilerCtx::Init) {
         SymbolType::Method
      } else {
         SymbolType::Func
      };

      // Declare the function in the function's parent's scope.
      let parent_symbol_pos = if !matches!(t, CompilerCtx::Lambda) {
         match self.declare_symbol(&decl.name, s_type) {
            Ok(p) => p,
            Err(_) => return,
         }
      } else {
         usize::MAX
      };

      // Change the compiler's context to a function or a method
      let prev_compiler_type = std::mem::replace(&mut self.compiler_type, t.clone());

      // The first element in a symbol table is always the symbol representing
      // the function to which the symbol table belongs, or the `self` variable.
      let symbols = SymbolTable::new(vec![match t {
         CompilerCtx::Method | CompilerCtx::Init => Symbol {
            name: String::from("self"),
            s_type: SymbolType::Class,
            is_initialized: true,
            depth: 0,
            is_used: true,
            line_info: func_pos,
            is_captured: false,
         },
         _ => Symbol {
            name: decl.name.lexeme.clone(),
            s_type: SymbolType::Func,
            is_initialized: true,
            depth: 0,
            is_used: true,
            line_info: func_pos,
            is_captured: false,
         },
      }]);

      // Make this function declaration the current function scope.
      self.functions.push(FunctionScope {
         function: FuncObject {
            defaults: vec![],
            min_arity: decl.arity.0,
            max_arity: decl.arity.1,
            chunk: Chunk::new(),
            name: decl.name.lexeme.clone(),
            up_val_count: 0,
         },
         s_table: symbols,
         scope_depth: 0,
         loops: vec![],
         breaks: vec![],
         up_values: vec![],
      });

      // compiles the parameters so that the compiler knows about their stack position.
      self.compile_parameters(&decl.params);

      // Compile the function's body
      if decl.body.is_empty() {
         self.emit_return(&None, func_pos, matches!(t, CompilerCtx::Init))
      } else {
         for (index, node) in decl.body.iter().enumerate() {
            self.compile_node(node);

            // Emit an implicit `return` if the body does not end with a return.
            if index == decl.body.len() - 1 {
               match *node {
                  ASTNode::ReturnStmt(_) => {}
                  _ => self.emit_return(&None, func_pos, matches!(t, CompilerCtx::Init)),
               }
            };
         }
      }

      // Pop all the symbols from the function's symbol table.
      self.current_s_table_mut().pop_scope(0, true, true);

      // Print the compiled function's chunk when the appropriate flag is on.
      #[cfg(feature = "show_bytecode")]
      self.print_pretty_bytecode();
      #[cfg(feature = "show_raw_bytecode")]
      self.print_raw_bytecode();

      // Takes the generated function object and up_values
      let function = std::mem::take(&mut self.current_func_scope_mut().function);
      let up_values = std::mem::take(&mut self.current_func_scope_mut().up_values);

      // Go back to the previous function.
      self.functions.pop();
      self.compiler_type = prev_compiler_type;

      // Loads the function object onto the stack at runtime.
      self.emit_function(function, up_values, &decl.name);

      if decl.arity.0 != decl.arity.1 {
         self.bind_default_params(decl);
      }

      if !matches!(t, CompilerCtx::Lambda) {
         if let CompilerCtx::Class = self.compiler_type {
            if self
               .add_literal_to_pool(Object::from(decl.name.lexeme.clone()), &decl.name, true)
               .is_some()
            {
               self.emit_op_code(OpCode::AppendClassField, func_pos);
            }
         }

         if self.is_global_scope() {
            self.define_as_global(&decl.name);
         }
         self.current_s_table_mut().mark_initialized(parent_symbol_pos);
      }
   }

   /// Emits the appropriate code to either load a function object from the constant or create
   /// a closure at runtime.
   ///
   /// # Parameters
   /// - `function`: The function object to be loaded.
   /// - `up_values`: The UpValues of this function.
   /// - `token`: A reference to the function's token.
   fn emit_function(&mut self, function: FuncObject, up_values: Vec<UpValue>, token: &Token) {
      let func = Object::from(function);
      let func_pos = (token.line_num, token.column_start);

      // If the function does not close over any values, then there is
      // no need to create a closure object at runtime.
      if up_values.is_empty() {
         self.add_literal_to_pool(func, token, true);
         return;
      }

      // Composes the closure object from within the parent function.
      if let Some(idx) = self.add_literal_to_pool(func, token, false) {
         if idx < 256 {
            if up_values.len() < 256 {
               self.emit_op_code_with_byte(OpCode::MakeClosure, idx as u8, func_pos);
            } else {
               self.emit_op_code_with_byte(OpCode::MakeClosureLarge, idx as u8, func_pos);
            }
         } else if up_values.len() < 256 {
            self.emit_op_code_with_short(OpCode::MakeClosureLong, idx, func_pos);
         } else {
            self.emit_op_code_with_short(OpCode::MakeClosureLongLarge, idx, func_pos);
         }

         self.emit_up_values(&up_values, func_pos);
      }
   }

   /// Emits the bytecode information required to compose the UpValues of a closure at runtime.
   ///
   /// # Parameters
   /// - `up_values`: The UpValues to be composed at runtime.
   /// - `func_pos`: The source position of the function declaration.
   fn emit_up_values(&mut self, up_values: &[UpValue], func_pos: (usize, usize)) {
      for up in up_values {
         // Emit the byte that the determines whether this up_value captures a local
         // variable or another up_value in the parent function.
         self.emit_raw_byte(if up.is_local { 1u8 } else { 0u8 }, func_pos);

         // Emit the byte or short for the position of the up_value.
         if up_values.len() < 256 {
            self.emit_raw_byte(up.index as u8, func_pos);
         } else {
            self.emit_raw_short(up.index as u16, func_pos);
         }
      }
   }

   /// Emits bytecode to bind the default values for the named parameters of a function.
   ///
   /// # Parameters
   /// * `decl`: The function-declaration node where these named parameters were declared.
   fn bind_default_params(&mut self, decl: &FunctionDeclNode) {
      // Compiles the named parameters so that they can be on top
      // of the stack when the function gets composed at runtime.
      for param in decl.params.iter() {
         match &param.default {
            Some(expr) => {
               self.compile_node(&expr);
            }
            None => {
               if param.is_optional {
                  self.emit_op_code(
                     OpCode::LoadImmNull,
                     (param.name.line_num, param.name.column_start),
                  );
               }
            }
         }
      }

      // Once all the named parameter expressions are compiled, we bind
      // each of the named parameters to the function
      self.emit_op_code_with_byte(
         OpCode::BindDefaults,
         (decl.arity.1 - decl.arity.0) as u8,
         (decl.name.line_num, decl.name.column_start),
      );
   }

   /// Compiles the parameter declaration statements of a function.
   pub(super) fn compile_parameters(&mut self, params: &[Parameter]) {
      for param in params.iter() {
         if self.declare_symbol(&param.name, SymbolType::Param).is_err() {
            // We do nothing if there was an error because the `declare_symbol()`
            // function takes care of reporting the appropriate error for us.
            // Explicit `return` to stop the loop.
            return;
         }
      }
   }

   /// Compiles a return statement.
   pub(super) fn compile_return_stmt(&mut self, stmt: &ReturnStmtNode) {
      if let CompilerCtx::Script = self.compiler_type {
         self.error_at_token(
            &stmt.token,
            CompilerErrorType::Syntax,
            "Cannot return outside of function.",
         );
         return;
      }

      if let CompilerCtx::Init = self.compiler_type {
         self.error_at_token(
            &stmt.token,
            CompilerErrorType::Syntax,
            "Cannot return from class initializer.",
         );
         return;
      }

      self.emit_return(&stmt.value, (stmt.token.line_num, stmt.token.column_start), false)
   }

   /// Emits bytecode to return out of a function at runtime.
   ///
   /// # Parameters
   /// - `value` (Option) – The AST node of the return expression (if any).
   /// - `token_pos`: The position of the return statement in the source code.
   fn emit_return(&mut self, value: &Option<Box<ASTNode>>, token_pos: (usize, usize), init: bool) {
      if init {
         self.emit_op_code_with_byte(OpCode::GetLocal, 0u8, token_pos);
      } else if let Some(node) = value {
         self.compile_node(node);
      } else {
         self.emit_op_code(OpCode::LoadImmNull, token_pos);
      }

      let depth = self.relative_scope_depth();
      let symbols = self.current_s_table_mut().pop_scope(depth, false, false);

      for (i, is_captured) in symbols.iter().rev().enumerate() {
         if *is_captured {
            if i < 256 {
               self.emit_op_code_with_byte(OpCode::CloseUpVal, i as u8, token_pos)
            } else {
               self.emit_op_code_with_short(OpCode::CloseUpValLong, i as u16, token_pos);
            }
         }
      }

      self.emit_op_code(OpCode::Return, token_pos);
   }
}
