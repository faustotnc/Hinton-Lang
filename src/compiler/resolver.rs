use crate::bytecode::OpCode;
use crate::compiler::symbols::{Symbol, SymbolType, SL};
use crate::compiler::{Compiler, UpValue};
use crate::errors::CompilerErrorType;
use crate::lexer::tokens::Token;
use crate::objects::Object;

impl Compiler {
   /// Looks for a symbol with the given token name in the current script.
   ///
   /// # Parameters
   /// - `token`: A reference to the token related to the symbol.
   /// - `reassign`: Whether we are resolving the symbol for the purpose of reassignment or not.
   ///
   /// # Returns
   /// - `Result<SL>`: The location (if found) and resolution type of the symbol.
   pub(super) fn resolve_symbol(&mut self, token: &Token, reassign: bool) -> Result<SL, ()> {
      // Look for the symbol in the local scope of the current function
      if let Some(s) = self.resolve_local(self.functions.len() - 1, token, reassign, None) {
         return Ok(s);
      }

      // If we are in a function within a block, then we also look for symbols
      // in the scope of the parent function to create upValues & closures.
      if self.functions.len() > 1 {
         if let Some(s) = self.resolve_up_value(token, reassign, self.functions.len() - 2) {
            return Ok(s);
         }
      }

      // Looks for the symbol in the global scope of the current script
      if let Some(s) = self.resolve_global(token, reassign) {
         return Ok(s);
      }

      // Look for the identifier in the natives
      if let Some(index) = self.natives.iter().position(|n| n == &token.lexeme) {
         if reassign {
            self.error_at_token(
               token,
               CompilerErrorType::Reassignment,
               &format!("Cannot modify native function '{}'.", token.lexeme),
            );

            return Ok(SL::Error);
         }

         self.emit_op_code_with_byte(
            OpCode::LoadNative,
            index as u8,
            (token.line_num, token.column_start),
         );

         return Ok(SL::Native);
      }

      // The symbol doesn't exist
      let error_msg = &format!("Use of undeclared identifier '{}'.", token.lexeme);
      self.error_at_token(token, CompilerErrorType::Reference, error_msg);
      Err(())
   }

   /// Looks for a symbol with the given token name in the local scope of the provided function index.
   ///
   /// # Parameters
   /// - `func_idx`: The index of the function in the compiler's function list where the variable
   ///    will be looked up.
   /// - `token`: A reference to the token related to the symbol.
   /// - `for_reassign`: Whether we are resolving the symbol for a reassignment or not.
   /// - `is_captured`: Whether this local variable has been captured by a closure or not. `None`
   ///    if it is unknown whether the variable is captured or not.
   ///
   /// # Returns
   /// - `Option<SL>`: The location (if found) and resolution type of the symbol.
   fn resolve_local(
      &mut self,
      func_idx: usize,
      token: &Token,
      for_reassignment: bool,
      is_captured: Option<bool>,
   ) -> Option<SL> {
      let func = &mut self.functions[func_idx];

      if let Some(resolution) = func.s_table.resolve(&token.lexeme, true, is_captured) {
         // Verify that the symbol has been initialized before using it.
         if self.check_is_initialized(&resolution, token).is_err() {
            return Some(SL::Error);
         }

         // If the resolution is for the purpose of reassignment, check that the symbol can be
         // reassigned before returning its location.
         return if self
            .check_reassignment(&resolution, token, for_reassignment)
            .is_ok()
         {
            Some(SL::Local(resolution.0, resolution.1))
         } else {
            Some(SL::Error)
         };
      }

      None
   }

   /// Looks for a symbol with the given token name in the global scope.
   ///
   /// # Parameters
   /// - `token`: A reference to the token (symbol name) related to the symbol.
   /// - `for_reassignment`: Whether we are resolving the symbol for the purpose of
   /// reassignment or not.
   ///
   /// # Returns
   /// - `Option<SL>`: The location (if found) and resolution type of the symbol.
   fn resolve_global(&mut self, token: &Token, for_reassignment: bool) -> Option<SL> {
      if let Some(resolution) = self.globals.resolve(&token.lexeme, true, None) {
         if self.check_is_initialized(&resolution, token).is_err() {
            return Some(SL::Error);
         }

         // If the resolution is for the purpose of reassignment, check that the symbol can be
         // reassigned before returning its location.
         if self
            .check_reassignment(&resolution, token, for_reassignment)
            .is_ok()
         {
            let name = Object::String(token.lexeme.clone());
            if let Some(idx) = self.add_literal_to_pool(name, &token, false) {
               return Some(SL::Global(resolution.0, idx as usize));
            }
         }

         return Some(SL::Error);
      }

      None
   }

   /// Checks that the resolved symbol has been initialized before usage.
   fn check_is_initialized(&mut self, s: &(Symbol, usize), token: &Token) -> Result<(), ()> {
      if s.0.is_initialized {
         return Ok(());
      }

      let sym_type = match s.0.s_type {
         SymbolType::Var => "variable",
         SymbolType::VarField => "variable class field",
         SymbolType::Const => "constant",
         SymbolType::ConstField => "constant class field",
         SymbolType::Func => "function",
         _ => unreachable!("Symbol should have been initialized by now."),
      };

      let error_msg = &format!(
         "Cannot reference {} '{}' before it has been initialized.",
         sym_type, token.lexeme
      );

      self.error_at_token(&token, CompilerErrorType::Reference, error_msg);
      Err(())
   }

   /// If the symbol has been resolved for the purpose of reassignment, this function
   /// makes sure that the symbol is reassignable.
   fn check_reassignment(&mut self, s: &(Symbol, usize), t: &Token, r: bool) -> Result<(), ()> {
      // If the symbol has not been resolved for reassignment in the first place,
      // simply return OK out of the function.
      if !r {
         return Ok(());
      }

      let message = match s.0.s_type {
         SymbolType::Const => "Constants are immutable.",
         SymbolType::Func => "Functions are immutable.",
         SymbolType::Class => "Classes are immutable.",
         SymbolType::ConstField => "Constant class fields are immutable.",
         SymbolType::Method => "Class methods are immutable.",
         // Only variables & parameters are re-assignable
         SymbolType::Var | SymbolType::VarField | SymbolType::Param => return Ok(()),
      };

      self.error_at_token(t, CompilerErrorType::Reassignment, message);
      Err(())
   }

   /// Looks for a symbol with the given token name in the provided function scope index.
   /// This function executes with the assumption that it is being called by a child
   /// function scope to look for UpValues in the scope of its parent, and will recursively
   /// look for the symbol in scopes of parent functions for provided function scope index.
   ///
   /// # Parameters
   /// - `token`: A reference to the token (symbol name) related to the symbol.
   /// - `reassign`: Whether we are resolving the symbol for the purpose of reassignment or not.
   /// - `func_idx`: The index of the function scope to start looking for the symbol.
   ///
   /// # Returns
   /// - `Option<SL>`: The location (if found) and resolution type of the symbol.
   fn resolve_up_value(&mut self, token: &Token, reassign: bool, func_idx: usize) -> Option<SL> {
      if func_idx == 0 && self.functions[0].scope_depth == 0 {
         return None;
      }

      // Look for the symbol in the local scope of the current function.
      // NOTE: By the time `self.resolve_up_value(...)` is called, the `self.resolve_symbol(...)`
      // function has already looked for local variables in the `current_func_scope`. So this
      // next call to `self.resolve_local_symbol(...)` is looking at symbols in the local scope
      // for the `self.functions.len() - 2` function scope. That is, the local scope of the parent
      // function of the parent function. Look at the call to `self.resolve_up_value(...)` in
      // `self.resolve_symbol(...)` to understand this better.
      if let Some(s) = self.resolve_local(func_idx, token, reassign, Some(true)) {
         return match s {
            SL::Local(s, p) => self.add_up_value(token, func_idx + 1, s, p, true),
            _ => unreachable!("SymbolLoc should have been a local symbol."),
         };
      }

      // Recursively look for the symbol in higher function scopes.
      if func_idx > 0 {
         if let Some(s) = self.resolve_up_value(token, reassign, func_idx - 1) {
            return match s {
               SL::UpValue(u, p) => self.add_up_value(token, func_idx + 1, u.symbol, p, false),
               _ => unreachable!("SymbolLoc should have been an up_value symbol."),
            };
         }
      }

      None
   }

   /// Adds an UpValue to the list of UpValues for the current function.
   fn add_up_value(
      &mut self,
      token: &Token,
      func_idx: usize,
      symbol: Symbol,
      index: usize,
      is_local: bool,
   ) -> Option<SL> {
      // Prevent creating repeated up_values
      for (index, up_val) in self.functions[func_idx].up_values.iter().enumerate() {
         if up_val.index == index && up_val.is_local == is_local && up_val.symbol.name == symbol.name {
            return Some(SL::UpValue(up_val.clone(), index));
         }
      }

      if self.functions[func_idx].up_values.len() >= u16::MAX as usize {
         self.error_at_token(
            token,
            CompilerErrorType::MaxCapacity,
            "Too many closure variables in function.",
         );
         return None;
      }

      let up_value = UpValue {
         symbol,
         index,
         is_local,
      };

      self.functions[func_idx].up_values.push(up_value.clone());
      self.functions[func_idx].function.up_val_count += 1;

      Some(SL::UpValue(
         up_value,
         self.functions[func_idx].up_values.len() - 1,
      ))
   }
}
