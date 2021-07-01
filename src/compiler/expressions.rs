use crate::ast::*;
use crate::bytecode::OpCode;
use crate::compiler::symbols::SL;
use crate::compiler::Compiler;
use crate::errors::CompilerErrorType;
use crate::lexer::tokens::Token;
use crate::objects::Object;

impl Compiler {
   /// Compiles a literal expression.
   pub(super) fn compile_literal_expr(&mut self, expr: &LiteralExprNode) {
      let obj = expr.value.clone();
      let opr_pos = (expr.token.line_num, expr.token.column_start);

      match obj {
         Object::Bool(x) if x => self.emit_op_code(OpCode::LoadImmTrue, opr_pos),
         Object::Bool(x) if !x => self.emit_op_code(OpCode::LoadImmFalse, opr_pos),
         Object::Int(x) if x == 0i64 => self.emit_op_code(OpCode::LoadImm0I, opr_pos),
         Object::Int(x) if x == 1i64 => self.emit_op_code(OpCode::LoadImm1I, opr_pos),
         Object::Float(x) if x == 0f64 => self.emit_op_code(OpCode::LoadImm0F, opr_pos),
         Object::Float(x) if x == 1f64 => self.emit_op_code(OpCode::LoadImm1F, opr_pos),
         Object::Null => self.emit_op_code(OpCode::LoadImmNull, opr_pos),

         // Compile integer literals with the immediate instruction when possible
         Object::Int(x) if x > 1i64 => {
            if x < 256i64 {
               self.emit_op_code_with_byte(OpCode::LoadImmN, x as u8, opr_pos);
            } else if x < (u16::MAX as i64) {
               self.emit_op_code_with_short(OpCode::LoadImmNLong, x as u16, opr_pos);
            } else {
               // If the number cannot be encoded within two bytes (as an unsigned short),
               // then we add it to the constant pool.
               self.add_literal_to_pool(obj, &expr.token, true);
            }
         }

         // If none of the above hold true, then load the literal from the pool at runtime.
         _ => {
            self.add_literal_to_pool(obj, &expr.token, true);
         }
      };
   }

   /// Compiles a unary expression.
   pub(super) fn compile_unary_expr(&mut self, expr: &UnaryExprNode) {
      self.compile_node(&expr.operand);

      let expression_op_code = match expr.opr_type {
         UnaryExprType::ArithmeticNeg => OpCode::Negate,
         UnaryExprType::LogicNeg => OpCode::LogicNot,
         UnaryExprType::BitwiseNeg => OpCode::BitwiseNot,
      };

      self.emit_op_code(expression_op_code, expr.pos);
   }

   /// Compiles a binary expression.
   pub(super) fn compile_binary_expr(&mut self, expr: &BinaryExprNode) {
      // Because logic 'OR' and logic 'AND' expressions are short-circuited,
      // they are compiled by a separate function.
      match expr.opr_type {
         BinaryExprType::LogicAND | BinaryExprType::LogicOR => {
            return self.compile_logic_and_or_expr(expr);
         }
         _ => {}
      }

      // Compiles the binary operators.
      self.compile_node(&expr.left);
      self.compile_node(&expr.right);

      let expr_op_code = match expr.opr_type {
         BinaryExprType::BitwiseAND => OpCode::BitwiseAnd,
         BinaryExprType::BitwiseOR => OpCode::BitwiseOr,
         BinaryExprType::BitwiseShiftLeft => OpCode::BitwiseShiftLeft,
         BinaryExprType::BitwiseShiftRight => OpCode::BitwiseShiftRight,
         BinaryExprType::BitwiseXOR => OpCode::BitwiseXor,
         BinaryExprType::Division => OpCode::Divide,
         BinaryExprType::Expo => OpCode::Expo,
         BinaryExprType::LogicAND => unreachable!("'AND' expressions not compiled here."),
         BinaryExprType::LogicEQ => OpCode::Equals,
         BinaryExprType::LogicGreaterThan => OpCode::GreaterThan,
         BinaryExprType::LogicGreaterThanEQ => OpCode::GreaterThanEq,
         BinaryExprType::LogicLessThan => OpCode::LessThan,
         BinaryExprType::LogicLessThanEQ => OpCode::LessThanEq,
         BinaryExprType::LogicNotEQ => OpCode::NotEq,
         BinaryExprType::LogicOR => unreachable!("'OR' expressions not compiled here."),
         BinaryExprType::Minus => OpCode::Subtract,
         BinaryExprType::Modulus => OpCode::Modulus,
         BinaryExprType::Multiplication => OpCode::Multiply,
         BinaryExprType::Nullish => OpCode::NullishCoalescing,
         BinaryExprType::Addition => OpCode::Add,
         BinaryExprType::Range => OpCode::MakeRange,
      };

      self.emit_op_code(
         expr_op_code,
         (expr.opr_token.line_num, expr.opr_token.column_start),
      );
   }

   /// Compiles a ternary conditional expression.
   /// This is compiled similarly to how if statements are compiled.
   pub(super) fn compile_ternary_conditional_expr(&mut self, expr: &TernaryConditionalNode) {
      self.compile_if_stmt(&IfStmtNode {
         condition: expr.condition.clone(),
         then_token: expr.true_branch_token.clone(),
         then_branch: expr.branch_true.clone(),
         else_branch: Box::new(Some(*expr.branch_false.clone())),
         else_token: Some(expr.false_branch_token.clone()),
      });
   }

   /// Compiles a logical 'AND' or a logical 'OR' expression.
   pub(super) fn compile_logic_and_or_expr(&mut self, expr: &BinaryExprNode) {
      // First compile the lhs of the expression which will leave its value on the stack.
      self.compile_node(&expr.left);

      let op_code = match expr.opr_type {
         // For 'AND' expressions, if the lhs is false, then the entire expression must be false.
         // We emit an `JUMP_IF_FALSE_OR_POP` instruction to jump over the rest of this
         // expression if the lhs is falsey.
         BinaryExprType::LogicAND => OpCode::JumpIfFalseOrPop,
         // For 'OR' expressions, if the lhs is true, then the entire expression must be true.
         // We emit an `JUMP_IF_TRUE_OR_POP` instruction to jump over to the next expression
         // if the lhs is truthy.
         BinaryExprType::LogicOR => OpCode::JumpIfTrueOrPop,
         // Other binary expressions are not allowed here.
         _ => unreachable!("Can only compile logic 'OR' or 'AND' expressions here."),
      };

      let end_jump = self.emit_jump(op_code, &expr.opr_token);
      self.compile_node(&expr.right);
      self.patch_jump(end_jump, &expr.opr_token);
   }

   /// Compiles an identifier expression.
   pub(super) fn compile_identifier_expr(&mut self, expr: &IdentifierExprNode) {
      if let Ok(res) = self.resolve_symbol(&expr.token, false) {
         self.named_variable(&res, &expr.token, false);
      }
   }

   /// Compiles a `self` expression.
   pub(super) fn compile_self_expr(&mut self, expr: &SelfExprNode) {
      if self.classes.is_empty() {
         self.error_at_token(
            &expr.token,
            CompilerErrorType::Reference,
            "Cannot use 'self' outside of class.",
         );

         return;
      }

      if let Ok(res) = self.resolve_symbol(&expr.token, false) {
         self.named_variable(&res, &expr.token, false);
      }
   }

   /// Emits the appropriate opcode to either get or set a local or global variable.
   ///
   /// # Parameters
   /// - `symbol`: The symbol and its location.
   /// - `token`: A reference to the token associated with this symbol.
   /// - `to_set`: Whether to emit reassignment instructions or not.
   fn named_variable(&mut self, symbol_loc: &SL, token: &Token, to_set: bool) {
      let pos = (token.line_num, token.column_start);

      let op_name;
      let op_name_long;
      let idx: usize;

      match symbol_loc {
         SL::Global(_, p) => {
            idx = *p;

            if to_set {
               op_name = OpCode::SetGlobal;
               op_name_long = OpCode::SetGlobalLong;
            } else {
               op_name = OpCode::GetGlobal;
               op_name_long = OpCode::GetGlobalLong;
            }
         }
         SL::Local(_, p) => {
            idx = *p;

            if to_set {
               op_name = OpCode::SetLocal;
               op_name_long = OpCode::SetLocalLong;
            } else {
               op_name = OpCode::GetLocal;
               op_name_long = OpCode::GetLocalLong;
            }
         }
         SL::UpValue(_, p) => {
            idx = *p;

            if to_set {
               op_name = OpCode::SetUpVal;
               op_name_long = OpCode::SetUpValLong;
            } else {
               op_name = OpCode::GetUpVal;
               op_name_long = OpCode::GetUpValLong;
            }
         }
         // Either there was an error with the resolution, or the symbol is for a native
         // function, so exit this here.
         _ => return,
      }

      if idx < 256 {
         self.emit_op_code_with_byte(op_name, idx as u8, pos);
      } else {
         self.emit_op_code_with_short(op_name_long, idx as u16, pos);
      }
   }

   /// Emits the appropriate instruction for a compound reassignment expression.
   ///
   /// # Parameters
   /// - `t`: The compound reassignment type
   /// - `line_info`: The source line info (line, column) of the reassignment expression.
   fn emit_compound_reassignment_opr(&mut self, t: &ReassignmentType, line_info: (usize, usize)) {
      match t {
         ReassignmentType::Plus => self.emit_op_code(OpCode::Add, line_info),
         ReassignmentType::Minus => self.emit_op_code(OpCode::Subtract, line_info),
         ReassignmentType::Div => self.emit_op_code(OpCode::Divide, line_info),
         ReassignmentType::Mul => self.emit_op_code(OpCode::Multiply, line_info),
         ReassignmentType::Expo => self.emit_op_code(OpCode::Expo, line_info),
         ReassignmentType::Mod => self.emit_op_code(OpCode::Modulus, line_info),
         ReassignmentType::ShiftL => self.emit_op_code(OpCode::BitwiseShiftLeft, line_info),
         ReassignmentType::ShiftR => self.emit_op_code(OpCode::BitwiseShiftRight, line_info),
         ReassignmentType::BitAnd => self.emit_op_code(OpCode::BitwiseAnd, line_info),
         ReassignmentType::Xor => self.emit_op_code(OpCode::BitwiseXor, line_info),
         ReassignmentType::BitOr => self.emit_op_code(OpCode::BitwiseOr, line_info),
         ReassignmentType::None => {}
      };
   }

   /// Compiles a variable reassignment expression.
   pub(super) fn compile_var_reassignment_expr(&mut self, expr: &VarReassignmentExprNode) {
      let line_info = (expr.target.line_num, expr.target.column_start);

      let res = match self.resolve_symbol(&expr.target, true) {
         Ok(symbol) => match symbol {
            SL::Global(s, p) => SL::Global(s, p),
            SL::Local(s, p) => SL::Local(s, p),
            SL::UpValue(u, p) => SL::UpValue(u, p),
            _ => return,
         },
         Err(()) => return,
      };

      if let ReassignmentType::None = expr.opr_type {
         // Proceed to directly reassign the variable.
         self.compile_node(&expr.value);
      } else {
         // The expression `a /= 2` expands to `a = a / 2`, so we
         // must get the variable's value onto the stack first.
         self.named_variable(&res, &expr.target, false);

         // Then we push the other operand's value onto the stack
         self.compile_node(&expr.value);

         // Then compute the operation of the two operands.
         self.emit_compound_reassignment_opr(&expr.opr_type, line_info);
      }

      // Sets the new value (which will be on top of the stack)
      self.named_variable(&res, &expr.target, true);
   }

   /// Compiles a property-reassignment expression.
   pub(super) fn compile_object_setter_expr(&mut self, expr: &ObjectSetExprNode) {
      self.compile_node(&expr.target);

      let prop_name = Object::String(expr.setter.lexeme.clone());
      let prop_line_info = (expr.setter.line_num, expr.setter.column_start);

      if let Some(pos) = self.add_literal_to_pool(prop_name, &expr.setter, false) {
         if let ReassignmentType::None = expr.opr_type {
            // Proceed to directly reassign the variable.
            self.compile_node(&expr.value);
         } else {
            // The expression `a.prop /= 2` expands to `a.prop = a.prop / 2`, so we
            // must get the property's value onto the stack first.
            self.compile_object_getter_expr(&ObjectGetExprNode {
               target: expr.target.clone(),
               getter: expr.setter.clone(),
            });

            // Then we push the other operand's value onto the stack
            self.compile_node(&expr.value);

            // Then compute the operation of the two operands.
            self.emit_compound_reassignment_opr(&expr.opr_type, prop_line_info);
         }

         if pos < 256 {
            self.emit_op_code_with_byte(OpCode::SetProp, pos as u8, prop_line_info);
         } else {
            self.emit_op_code_with_short(OpCode::SetPropLong, pos, prop_line_info);
         }
      }
   }

   /// Compiles an object property access expression.
   pub(super) fn compile_object_getter_expr(&mut self, expr: &ObjectGetExprNode) {
      self.compile_node(&expr.target);

      let prop_name = Object::String(expr.getter.lexeme.clone());
      let prop_line_info = (expr.getter.line_num, expr.getter.column_start);

      if let Some(pos) = self.add_literal_to_pool(prop_name, &expr.getter, false) {
         if pos < 256 {
            self.emit_op_code_with_byte(OpCode::GetProp, pos as u8, prop_line_info);
         } else {
            self.emit_op_code_with_short(OpCode::GetPropLong, pos, prop_line_info);
         }
      }
   }

   /// Compiles an array literal expression.
   pub(super) fn compile_array_expr(&mut self, expr: &ArrayExprNode) {
      if expr.values.len() <= (u16::MAX as usize) {
         let line_info = (expr.token.line_num, expr.token.column_start);

         // We reverse the list here because at runtime, we pop each value of the stack in the
         // opposite order (because it *is* a stack). Instead of performing that operation during
         // runtime, we execute it once during compile time.
         for node in expr.values.iter().rev() {
            self.compile_node(&node);
         }

         if expr.values.len() < 256 {
            self.emit_op_code_with_byte(OpCode::MakeArray, expr.values.len() as u8, line_info);
         } else {
            self.emit_op_code_with_short(OpCode::MakeArrayLong, expr.values.len() as u16, line_info);
         }
      } else {
         self.error_at_token(
            &expr.token,
            CompilerErrorType::MaxCapacity,
            "Too many values in the array.",
         );
      }
   }

   /// Compiles a tuple literal expression.
   pub(super) fn compile_tuple_expr(&mut self, expr: &TupleExprNode) {
      if expr.values.len() <= (u16::MAX as usize) {
         let line_info = (expr.token.line_num, expr.token.column_start);

         // We reverse the list here because at runtime, we pop each value of the stack in the
         // opposite order (because it *is* a stack). Instead of performing that operation during
         // runtime, we execute it once during compile time.
         for node in expr.values.iter().rev() {
            self.compile_node(&node);
         }

         if expr.values.len() < 256 {
            self.emit_op_code(OpCode::MakeTuple, line_info);
            self.emit_raw_byte(expr.values.len() as u8, line_info);
         } else {
            self.emit_op_code(OpCode::MakeTupleLong, line_info);
            self.emit_raw_short(expr.values.len() as u16, line_info);
         }
      } else {
         self.error_at_token(
            &expr.token,
            CompilerErrorType::MaxCapacity,
            "Too many values in the tuple.",
         );
      }
   }

   /// Compiles a dictionary literal expression.
   pub(super) fn compile_dictionary(&mut self, expr: &DictionaryExprNode) {
      let pair_count = expr.keys.len();
      let pos = (expr.token.line_num, expr.token.column_start);

      // Prevent having too many key-value pairs for the dictionary.
      if pair_count > u16::MAX as usize {
         self.error_at_token(
            &expr.token,
            CompilerErrorType::MaxCapacity,
            "Too many key-value pairs for the dictionary.",
         );

         return;
      }

      // Compile the key-value pairs and leave them on the stack
      for (key, value) in expr.keys.iter().zip(expr.values.iter()) {
         // Get the key name from the token
         let name = if key.lexeme.starts_with('\"') {
            let lexeme = key.lexeme.clone();

            // Remove outer quotes from the source string
            let lexeme = &lexeme[1..(lexeme.len() - 1)];

            // Replace escaped characters with the actual representations
            lexeme
               .replace("\\n", "\n")
               .replace("\\t", "\t")
               .replace("\\r", "\r")
               .replace("\\\\", "\\")
               .replace("\\\"", "\"")
         } else {
            key.lexeme.clone()
         };

         self.add_literal_to_pool(Object::String(name), key, true);
         self.compile_node(value);
      }

      // Emit instruction to make the dictionary at runtime
      if pair_count < 256 {
         self.emit_op_code_with_byte(OpCode::MakeDict, pair_count as u8, pos);
      } else {
         self.emit_op_code_with_short(OpCode::MakeDictLong, pair_count as u16, pos);
      }
   }

   /// Compiles an array indexing expression.
   pub(super) fn compile_array_indexing_expr(&mut self, expr: &ArrayIndexingExprNode) {
      self.compile_node(&expr.target);
      self.compile_node(&expr.index);
      self.emit_op_code(OpCode::Indexing, expr.pos);
   }

   /// Compiles a function call or new instance expression.
   pub(super) fn compile_inst_or_func_call_expr(&mut self, expr: &FunctionCallExprNode, inst: bool) {
      // Compile the call's identifier
      self.compile_node(&expr.target);

      // Compile call's arguments
      for arg in expr.args.iter() {
         self.compile_node(&arg.value);
      }

      // Call the function or create an instance at runtime
      if inst {
         self.emit_op_code_with_byte(OpCode::MakeInstance, expr.args.len() as u8, expr.pos);
      } else {
         self.emit_op_code_with_byte(OpCode::FuncCall, expr.args.len() as u8, expr.pos);
      }
   }
}
