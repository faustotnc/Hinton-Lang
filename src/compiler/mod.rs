use crate::ast::{ASTNode, ModuleNode};
use crate::bytecode::{Chunk, OpCode};
use crate::compiler::symbols::{Symbol, SymbolTable, SymbolType};
use crate::errors::{CompilerErrorType, ErrorReport};
use crate::lexer::tokens::Token;
use crate::objects::{FuncObject, Object};
use std::convert::TryFrom;
use std::path::Path;

// Submodules
mod expressions;
mod functions;
mod loops;
mod resolver;
mod statements;
mod symbols;

/// Represents a break statement, which is associated with a loop.
struct BreakScope {
   /// The loop scope associated with this break statement.
   parent_loop: LoopScope,
   /// The position of the break's instruction in the chunk.
   chunk_pos: usize,
}

/// Represents a loop statement at compile time. Used primarily by
/// break statements to know which loops to break.
#[derive(Clone, Copy)]
struct LoopScope {
   /// The position of the loop's start in the chunk.
   position: usize,
   /// The scope depth of the body of the loop.
   scope_depth: usize,
   /// The type of loop this represents.
   loop_type: LoopType,
}

/// The types of loops available in Hinton. This is useful when \
/// compiling break statements to emit extra `POP` operations \
/// based on the loop type.
#[derive(Clone, Copy)]
enum LoopType {
   ForIn,
   While,
}

/// Represents the context that the compiler is currently using
/// to compile the AST.
#[derive(Clone)]
enum CompilerCtx {
   Class,
   Function,
   Init,
   Method,
   Script,
}

/// A special type of scope for compiling function declarations.
pub struct FunctionScope {
   /// The function object where the bytecode will be emitted
   function: FuncObject,
   /// UpValues (free variables) captured by this function.
   up_values: Vec<UpValue>,
   /// Lexical scoping of declarations made inside the function's body.
   s_table: SymbolTable,
   /// The scope depth of declarations made inside the function's body,
   /// relative to the function's scope depth.
   scope_depth: usize,
   /// Declaration of loops made inside this function, where the last
   /// element of this vector represents the inner-most loop currently
   /// being compiled.
   loops: Vec<LoopScope>,
   /// A list of break statements made inside a loop.
   breaks: Vec<BreakScope>,
}

#[derive(Clone)]
/// Represents an UpValue (free variable) in a closure.
pub struct UpValue {
   /// The captured symbol.
   pub symbol: Symbol,
   /// The index of the symbol in the parent function.
   pub index: usize,
   /// Whether this UpValue refers to a local variable
   /// in a parent function, or an UpValue captured by
   /// the parent function.
   pub is_local: bool,
}

pub struct ClassScope {
   members: SymbolTable,
}

/// Represents the compiler and its internal state.
pub struct Compiler {
   /// A list of function declarations made inside the program,
   /// where the last element of this vector represents the
   /// inner-most function currently being compiled.
   functions: Vec<FunctionScope>,
   /// A symbol table for the global declarations made in the program.
   globals: SymbolTable,
   /// A list of string names of Hinton native functions.
   natives: Vec<String>,
   /// A list of compiler errors generated while compiling the program.
   errors: Vec<ErrorReport>,
   /// The type of chunk currently being compiled.
   compiler_type: CompilerCtx,
   /// A list of class declarations made inside the program, where
   /// the last element of this vector represents the inner-most
   /// class declaration currently being compiled.
   classes: Vec<ClassScope>,
}

impl Compiler {
   /// Compiles an Abstract Syntax Tree into ByteCode.
   ///
   /// # Parameters
   /// - `filepath`: The program's filepath.
   /// - `program`: The root node of the AST for a particular program.
   /// - `natives`: A list of string names of Hinton native functions.
   ///
   /// # Returns
   /// `Result<chunk, InterpretResult>`: If the program had no compile-time errors, returns
   /// the main chunk for this module. Otherwise, returns an
   /// InterpretResult::INTERPRET_COMPILE_ERROR.
   pub fn compile_ast(
      filepath: &Path,
      program: &ASTNode,
      natives: Vec<String>,
   ) -> Result<FuncObject, Vec<ErrorReport>> {
      // The first element in a symbol table is always the symbol representing
      // the function to which the symbol table belongs.
      let symbols = SymbolTable::new(vec![Symbol {
         name: format!("<File '{}'>", filepath.to_str().unwrap()),
         s_type: SymbolType::Func,
         is_initialized: true,
         depth: 0,
         is_used: true,
         line_info: (0, 0),
         is_captured: false,
      }]);

      let base_fn = FunctionScope {
         function: FuncObject {
            defaults: vec![],
            min_arity: 0,
            max_arity: 0,
            chunk: Chunk::new(),
            name: format!("<File '{}'>", filepath.to_str().unwrap()),
            up_val_count: 0,
         },
         s_table: symbols,
         scope_depth: 0,
         loops: vec![],
         breaks: vec![],
         up_values: vec![],
      };

      let mut _self = Compiler {
         compiler_type: CompilerCtx::Script,
         functions: vec![base_fn],
         errors: vec![],
         globals: SymbolTable::new(vec![]),
         natives,
         classes: vec![],
      };

      // Compile the function body
      _self.compile_node(&program);
      _self.emit_op_code(OpCode::EndVirtualMachine, (0, 0));

      // Print the bytecode for the main function when the appropriate flag is present.
      #[cfg(feature = "show_bytecode")]
      _self.print_pretty_bytecode();
      #[cfg(feature = "show_raw_bytecode")]
      _self.print_raw_bytecode();

      if _self.errors.is_empty() {
         Ok(std::mem::take(&mut _self.current_func_scope_mut().function))
      } else {
         Err(_self.errors)
      }
   }

   /// Compiles an AST node.
   fn compile_node(&mut self, node: &ASTNode) {
      match node {
         ASTNode::Array(x) => self.compile_array_expr(x),
         ASTNode::ArrayIndexing(x) => self.compile_array_indexing_expr(x),
         ASTNode::Binary(x) => self.compile_binary_expr(x),
         ASTNode::BlockStmt(x) => self.compile_block_stmt(x),
         ASTNode::ClassDecl(x) => self.compile_class_declaration(x),
         ASTNode::ConstantDecl(x) => self.compile_constant_decl(x),
         ASTNode::Dictionary(x) => self.compile_dictionary(x),
         ASTNode::ExpressionStmt(x) => self.compile_expression_stmt(x),
         ASTNode::ForStmt(x) => self.compile_for_stmt(x),
         ASTNode::FunctionCall(x) => self.compile_inst_or_func_call_expr(x, false),
         ASTNode::FunctionDecl(x) => self.compile_function_decl(x, CompilerCtx::Function),
         ASTNode::Identifier(x) => self.compile_identifier_expr(x),
         ASTNode::IfStmt(x) => self.compile_if_stmt(x),
         ASTNode::Instance(x) => self.compile_inst_or_func_call_expr(x, true),
         ASTNode::Literal(x) => self.compile_literal_expr(x),
         ASTNode::LoopBranch(x) => self.compile_loop_branching_stmt(x),
         ASTNode::Module(x) => self.compile_module_node(x),
         ASTNode::ObjectGetter(x) => self.compile_object_getter_expr(x),
         ASTNode::ObjectSetter(x) => self.compile_object_setter_expr(x),
         ASTNode::ReturnStmt(x) => self.compile_return_stmt(x),
         ASTNode::TernaryConditional(x) => self.compile_ternary_conditional_expr(x),
         ASTNode::Tuple(x) => self.compile_tuple_expr(x),
         ASTNode::Unary(x) => self.compile_unary_expr(x),
         ASTNode::VarReassignment(x) => self.compile_var_reassignment_expr(x),
         ASTNode::VariableDecl(x) => self.compile_variable_decl(x),
         ASTNode::WhileStmt(x) => self.compile_while_stmt(x),
         ASTNode::SelfExpr(x) => self.compile_self_expr(x),
      }
   }

   /// Compiles an AST module node.
   fn compile_module_node(&mut self, module: &ModuleNode) {
      for node in module.body.iter() {
         self.compile_node(node);
      }
   }

   /// Checks that the compiler is currently in the global scope.
   fn is_global_scope(&self) -> bool {
      if let CompilerCtx::Script = self.compiler_type {
         if self.functions.len() == 1 && self.relative_scope_depth() == 0 {
            return true;
         }
      }

      false
   }

   /// Gets an immutable reference to the current function scope.
   fn current_func_scope(&self) -> &FunctionScope {
      self.functions.last().unwrap()
   }

   /// Gets a mutable reference to the current function scope.
   fn current_func_scope_mut(&mut self) -> &mut FunctionScope {
      self.functions.last_mut().unwrap()
   }

   /// Gets an immutable reference to the current class scope.
   fn current_class_scope(&self) -> Option<&ClassScope> {
      self.classes.last()
   }

   /// Gets a mutable reference to the current class scope.
   fn current_class_scope_mut(&mut self) -> Option<&mut ClassScope> {
      self.classes.last_mut()
   }

   fn current_s_table(&self) -> &SymbolTable {
      if self.is_global_scope() {
         return &self.globals;
      }

      if let CompilerCtx::Class = self.compiler_type {
         &self.current_class_scope().unwrap().members
      } else {
         &self.current_func_scope().s_table
      }
   }

   fn current_s_table_mut(&mut self) -> &mut SymbolTable {
      if self.is_global_scope() {
         return &mut self.globals;
      }

      if let CompilerCtx::Class = self.compiler_type {
         &mut self.current_class_scope_mut().unwrap().members
      } else {
         &mut self.current_func_scope_mut().s_table
      }
   }

   /// Gets an immutable reference to the chunk where the compiler
   /// is currently emitting bytecode into.
   fn current_chunk(&self) -> &Chunk {
      &self.current_func_scope().function.chunk
   }

   /// Gets a mutable reference to the chunk where the compiler
   /// is currently emitting bytecode into.
   fn current_chunk_mut(&mut self) -> &mut Chunk {
      &mut self.current_func_scope_mut().function.chunk
   }

   /// Gets the current scope depth relative to the current function scope.
   fn relative_scope_depth(&self) -> usize {
      self.current_func_scope().scope_depth
   }

   /// Pretty-prints the compiled chunk of bytecode fot the current function.
   #[cfg(feature = "show_bytecode")]
   fn print_pretty_bytecode(&self) {
      crate::bytecode::disassemble_function_scope(
         &self.current_func_scope().function.chunk,
         &self.natives,
         &self.current_func_scope().function.name,
      );
   }

   /// Prints the raw bytes compiled into the current function's chunk.
   #[cfg(feature = "show_raw_bytecode")]
   fn print_raw_bytecode(&self) {
      crate::bytecode::print_raw(
         self.current_chunk(),
         self.current_func_scope().function.name.clone().as_str(),
      );
   }

   /// Emits a byte instruction from an OpCode into the chunk's instruction list.
   ///
   /// # Parameters
   /// - `instr`: The OpCode instruction to be added to the chunk.
   /// - `pos`: The source line and column associated with this OpCode.
   fn emit_op_code(&mut self, instr: OpCode, pos: (usize, usize)) {
      self.current_chunk_mut().push_op_code(instr);
      self.current_chunk_mut().push_line_info(pos);
   }

   /// Emits a raw byte instruction into the chunk's instruction list.
   ///
   /// # Parameters
   /// - `byte`: The byte instruction to be added to the chunk.
   /// - `pos`: The source line and column associated with this raw byte.
   fn emit_raw_byte(&mut self, byte: u8, pos: (usize, usize)) {
      self.current_chunk_mut().push_byte(byte);
      self.current_chunk_mut().push_line_info(pos);
   }

   /// Emits a raw short instruction from a 16-bit integer into the chunk's instruction list.
   ///
   /// # Parameters
   /// - `instr`: The 16-bit short instruction to add to the chunk.
   /// - `pos`: The source line and column associated with this raw short.
   fn emit_raw_short(&mut self, short: u16, pos: (usize, usize)) {
      self.current_chunk_mut().push_short(short);
      self.current_chunk_mut().push_line_info(pos);
      self.current_chunk_mut().push_line_info(pos);
   }

   /// Emits a byte instruction from an OpCode into the chunk's instruction list, followed by a raw byte.
   ///
   /// # Parameters
   /// - `instr`: The OpCode instruction to be added to the chunk.
   /// - `byte`: The raw byte to follow the emitted instruction.
   /// - `pos`: The source line and column associated with this OpCode.
   fn emit_op_code_with_byte(&mut self, instr: OpCode, byte: u8, pos: (usize, usize)) {
      self.emit_op_code(instr, pos);
      self.emit_raw_byte(byte, pos);
   }

   /// Emits a byte instruction from an OpCode into the chunk's instruction list, followed by two more
   /// raw bytes from a 16-bit integer.
   ///
   /// # Parameters
   /// - `instr`: The OpCode short instruction to be added to the chunk.
   /// - `short`: The 16-bit short instruction to add to the chunk.
   /// - `pos`: The source line and column associated with this OpCode.
   fn emit_op_code_with_short(&mut self, instr: OpCode, short: u16, pos: (usize, usize)) {
      self.emit_op_code(instr, pos);
      self.emit_raw_short(short, pos);
   }

   /// Emits a jump instructions with a dummy jump offset. This offset should be
   /// later replaced by calling the `patch_jump(...)` function.
   ///
   /// # Parameters
   /// - `instruction`: The jump instruction to emit to the chunk.
   /// - `token`: The token associated with this jump.
   ///
   /// # Returns
   /// `usize`: The position of the currently emitted jump instruction in the chunk. This value
   /// should be used by the call to the `patch_jump(...)` function to patch the correct jump
   /// instruction's offset.
   fn emit_jump(&mut self, instruction: OpCode, token: &Token) -> usize {
      self.emit_op_code_with_short(instruction, 0xffff, (token.line_num, token.column_start));
      return self.current_chunk_mut().len() - 2;
   }

   /// Patches the offset of a jump instruction.
   ///
   /// # Parameters
   /// - `offset`: The position in the chunk of the jump instruction to be patched.
   /// - `token`: The token associated with this jump patch.
   fn patch_jump(&mut self, offset: usize, token: &Token) {
      // -2 to adjust for the bytecode for the jump offset itself.
      match u16::try_from(self.current_chunk_mut().len() - offset - 2) {
         Ok(forward) => {
            let jump = forward.to_be_bytes();

            self.current_chunk_mut().modify_byte(offset, jump[0]);
            self.current_chunk_mut().modify_byte(offset + 1, jump[1]);
         }
         Err(_) => self.error_at_token(
            token,
            CompilerErrorType::MaxCapacity,
            "Too much code to jump over.",
         ),
      }
   }

   /// Emits a `LoopJump` or `LoopJumpLong` instruction.
   ///
   /// # Parameters
   /// - `loop_start`: The position in the chunk of the loop's first instruction.
   /// - `token`: The token associated with this loop instruction.
   fn emit_loop(&mut self, loop_start: usize, token: &Token) {
      let offset = self.current_chunk_mut().len() - loop_start;
      let line_info = (token.line_num, token.column_start);

      if offset < (u8::MAX - 2) as usize {
         self.emit_op_code(OpCode::LoopJump, line_info);

         // +2 to account for the 'OP_LOOP_JUMP' and its operand.
         let jump = (offset + 2) as u8;
         self.emit_raw_byte(jump, line_info);
      } else if offset < (u16::MAX - 3) as usize {
         self.emit_op_code(OpCode::LoopJumpLong, line_info);

         // +3 to account for the 'OP_LOOP_JUMP_LONG' and its operands.
         let jump = (offset + 3) as u16;
         self.emit_raw_short(jump, line_info);
      } else {
         self.error_at_token(token, CompilerErrorType::MaxCapacity, "Loop body too large.")
      }
   }

   /// Adds an object to the pool and (optionally) emits a `LoadConst` or `LoadConstLong` instruction.
   ///
   /// # Arguments
   /// - `obj`: The literal object to be added to the pool.
   /// - `token`: The object's original token.
   /// - `load`: Whether we should also emit a `LoadConst` or `LoadConstLong` instruction or not.
   ///
   /// # Returns
   /// `Option<u16>`: The position of this object in the pool. Returns `None` if the pool is fool.
   pub fn add_literal_to_pool(&mut self, obj: Object, token: &Token, load: bool) -> Option<u16> {
      let constant_pos = self.current_chunk_mut().add_constant(obj);
      let opr_pos = (token.line_num, token.column_start);

      match constant_pos {
         Ok(idx) => {
            if load {
               if idx < 256 {
                  self.emit_op_code(OpCode::LoadConstant, opr_pos);
                  self.emit_raw_byte(idx as u8, opr_pos);
               } else {
                  self.emit_op_code(OpCode::LoadConstantLong, opr_pos);
                  self.emit_raw_short(idx, opr_pos);
               }
            }

            Some(idx)
         }
         Err(_) => {
            self.error_at_token(
               token,
               CompilerErrorType::MaxCapacity,
               "Too many constants in one chunk.",
            );

            None
         }
      }
   }

   /// Emits a compiler error from the given token.
   ///
   /// # Parameters
   /// - `token`: The token that caused the error.
   /// - `err_type`: The type of error to be emitted.
   /// - `message`: The error message to display.
   fn error_at_token(&mut self, token: &Token, err_type: CompilerErrorType, message: &str) {
      let err_name = match err_type {
         CompilerErrorType::MaxCapacity => "MaxCapacityError",
         CompilerErrorType::Reassignment => "ReassignmentError",
         CompilerErrorType::Reference => "ReferenceError",
         CompilerErrorType::Syntax => "SyntaxError",
         CompilerErrorType::Duplication => "DuplicationError",
      };

      let msg = format!(
         "\x1b[31;1m{}\x1b[0m\x1b[1m at [{}:{}]: {}\x1b[0m",
         err_name, token.line_num, token.column_start, message
      );

      self.errors.push(ErrorReport {
         line: token.line_num,
         column: token.column_start,
         lexeme_len: token.lexeme.len(),
         message: msg,
      });
   }
}
