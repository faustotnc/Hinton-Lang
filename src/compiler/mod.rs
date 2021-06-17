use self::symbols::{Symbol, SymbolTable, SymbolType};
use crate::{
    ast::*,
    bytecode::{self, Chunk, OpCode},
    errors::CompilerErrorType,
    errors::ErrorReport,
    lexer::tokens::Token,
    objects::{FuncObject, Object},
};
use std::{convert::TryFrom, str, vec};

// Submodules
mod expressions;
mod functions;
mod loops;
mod resolver;
mod statements;
mod symbols;

/// Represents a break statement, which is associated with a loop.
struct BreakScope {
    parent_loop: LoopScope,
    loop_position: usize,
}

/// Represents a loop statement at compile time. Used primarily by
/// break statements to know which loops to break.
#[derive(Clone, Copy)]
struct LoopScope {
    /// The position of the loop's start in the chunk
    position: usize,
    /// The scope depth of the body of the loop
    scope_depth: usize,
    /// The type of loop this represents
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

/// Types of compilers we can create
#[derive(Clone)]
enum CompilerType {
    Function,
    Script,
}

pub struct FunctionScope {
    function: FuncObject,
    up_values: Vec<UpValue>,
    // Lexical scoping of declarations
    s_table: SymbolTable,
    scope_depth: usize,
    // Used to match the break statements with the correct
    // loop (specially useful with nested loops), and to
    // patch the break (OP_JUMP) offsets.
    loops: Vec<LoopScope>,
    breaks: Vec<BreakScope>,
}

#[derive(Clone)]
/// Represents an UpValue (free variable) in a closure.
pub struct UpValue {
    /// The captured symbol.
    pub symbol: Symbol,
    /// The index of the symbol in the parent function.
    pub index: usize,
    /// Whether or not this UpValue refers to a local
    /// variable in a parent function, or an UpValue
    /// captured by the parent function.
    pub is_local: bool,
}

/// Represents a compiler and its internal state.
pub struct Compiler {
    compiler_type: CompilerType,
    functions: Vec<FunctionScope>,
    globals: SymbolTable,
    natives: Vec<String>,
    errors: Vec<ErrorReport>,
}

impl Compiler {
    /// Compiles an Abstract Syntax Tree into ByteCode.
    /// While the AST is used to perform static analysis like type checking
    /// and error detection, the ByteCode is used to execute the program faster.
    ///
    /// ## Arguments
    /// * `filepath` – The program's filepath.
    /// * `program` – The root node of the AST for a particular program.
    ///
    /// ## Returns
    /// `Result<chunk, InterpretResult>` – If the program had no compile-time errors, returns
    /// the main chunk for this module. Otherwise returns an InterpretResult::INTERPRET_COMPILE_ERROR.
    pub fn compile_file(
        filepath: &str,
        program: &ASTNode,
        natives: Vec<String>,
    ) -> Result<FuncObject, Vec<ErrorReport>> {
        // The first element in a symbol table is always the symbol representing
        // the function to which the symbol table belongs.
        let symbols = SymbolTable::new(vec![Symbol {
            name: format!("<File '{}'>", filepath),
            symbol_type: SymbolType::Function,
            is_initialized: true,
            symbol_depth: 0,
            is_used: true,
            line_info: (0, 0),
            is_captured: false,
        }]);

        let base_fn = FunctionScope {
            function: FuncObject {
                defaults: vec![],
                min_arity: 0,
                max_arity: 0,
                chunk: bytecode::Chunk::new(),
                name: format!("<File '{}'>", filepath),
                up_val_count: 0,
            },
            s_table: symbols,
            scope_depth: 0,
            loops: vec![],
            breaks: vec![],
            up_values: vec![],
        };

        let mut _self = Compiler {
            compiler_type: CompilerType::Script,
            functions: vec![base_fn],
            errors: vec![],
            globals: SymbolTable::new(vec![]),
            natives,
        };

        // Compile the function body
        _self.compile_node(&program);
        _self.emit_op_code(bytecode::OpCode::EndVirtualMachine, (0, 0));

        // Print the bytecode for the main function when the appropriate flag is set.
        #[cfg(feature = "show_bytecode")]
        _self.print_pretty_bytecode();
        #[cfg(feature = "show_raw_bytecode")]
        _self.print_raw_bytecode();

        if _self.errors.len() == 0 {
            Ok(std::mem::take(&mut _self.current_func_scope_mut().function))
        } else {
            return Err(_self.errors);
        }
    }

    /// Compiles an AST node.
    ///
    /// ## Arguments
    /// * `node` – The node to be compiled.
    fn compile_node(&mut self, node: &ASTNode) {
        return match node {
            ASTNode::Array(x) => self.compile_array_expr(x),
            ASTNode::ArrayIndexing(x) => self.compile_array_indexing_expr(x),
            ASTNode::Binary(x) => self.compile_binary_expr(x),
            ASTNode::BlockStmt(x) => self.compile_block_stmt(x),
            ASTNode::ClassDecl(x) => self.compile_class_declaration(x),
            ASTNode::ConstantDecl(x) => self.compile_constant_decl(x),
            ASTNode::Dictionary(x) => self.compile_dictionary(x),
            ASTNode::ExpressionStmt(x) => self.compile_expression_stmt(x),
            ASTNode::ForStmt(x) => self.compile_for_stmt(x),
            ASTNode::FunctionCall(x) => self.compile_instance_or_func_call_expr(x, false),
            ASTNode::FunctionDecl(x) => self.compile_function_decl(x),
            ASTNode::Identifier(x) => self.compile_identifier_expr(x),
            ASTNode::IfStmt(x) => self.compile_if_stmt(x),
            ASTNode::Instance(x) => self.compile_instance_or_func_call_expr(x, true),
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
        };
    }

    fn compile_module_node(&mut self, module: &ModuleNode) {
        for node in module.body.iter() {
            self.compile_node(node);
        }
    }

    fn is_global_scope(&self) -> bool {
        if self.functions.len() == 1 && self.relative_scope_depth() == 0 {
            if let CompilerType::Script = self.compiler_type {
                return true;
            }
        }

        false
    }

    fn current_function_scope(&self) -> &FunctionScope {
        self.functions.last().unwrap()
    }

    fn current_func_scope_mut(&mut self) -> &mut FunctionScope {
        self.functions.last_mut().unwrap()
    }

    fn current_chunk(&self) -> &Chunk {
        &self.current_function_scope().function.chunk
    }

    fn current_chunk_mut(&mut self) -> &mut Chunk {
        &mut self.current_func_scope_mut().function.chunk
    }

    fn relative_scope_depth(&self) -> usize {
        self.current_function_scope().scope_depth
    }

    #[cfg(feature = "show_bytecode")]
    fn print_pretty_bytecode(&self) {
        bytecode::disassemble_function_scope(
            &self.current_function_scope().function.chunk,
            &self.natives,
            &self.current_function_scope().function.name,
        );
    }

    #[cfg(feature = "show_raw_bytecode")]
    fn print_raw_bytecode(&self) {
        bytecode::print_raw(
            self.current_chunk(),
            self.current_function_scope().function.name.clone().as_str(),
        );
    }

    /// Emits a byte instruction from an OpCode into the chunk's instruction list.
    ///
    /// ## Arguments
    /// * `instr` – The OpCode instruction to added to the chunk.
    /// * `pos` – The source line and column associated with this op_code.
    fn emit_op_code(&mut self, instr: bytecode::OpCode, pos: (usize, usize)) {
        self.current_chunk_mut().push_op_code(instr);
        self.current_chunk_mut().push_line_info(pos);
    }

    /// Emits a raw byte instruction into the chunk's instruction list.
    ///
    /// ## Arguments
    /// * `byte` – The byte instruction to added to the chunk.
    /// * `pos` – The source line and column associated with this op_code.
    fn emit_raw_byte(&mut self, byte: u8, pos: (usize, usize)) {
        self.current_chunk_mut().push_byte(byte);
        self.current_chunk_mut().push_line_info(pos);
    }

    /// Emits a short instruction from a 16-bit integer into the chunk's instruction list.
    ///
    /// ## Arguments
    /// * `instr` – The 16-bit short instruction to added to the chunk.
    /// * `pos` – The source line and column associated with this op_code.
    fn emit_short(&mut self, instr: u16, pos: (usize, usize)) {
        self.current_chunk_mut().push_short(instr);
        self.current_chunk_mut().push_line_info(pos);
        self.current_chunk_mut().push_line_info(pos);
    }

    fn emit_op_code_with_byte(&mut self, instr: bytecode::OpCode, byte: u8, pos: (usize, usize)) {
        self.current_chunk_mut().push_op_code(instr);
        self.current_chunk_mut().push_byte(byte);
        self.current_chunk_mut().push_line_info(pos);
        self.current_chunk_mut().push_line_info(pos);
    }

    fn emit_op_code_with_short(&mut self, instr: bytecode::OpCode, short: u16, pos: (usize, usize)) {
        self.current_chunk_mut().push_op_code(instr);
        self.current_chunk_mut().push_short(short);
        self.current_chunk_mut().push_line_info(pos);
        self.current_chunk_mut().push_line_info(pos);
        self.current_chunk_mut().push_line_info(pos);
    }

    /// Emits a jump instructions with a dummy jump offset. This offset should be
    // later replaced by calling the `patch_jump(...)` function.
    ///
    /// ## Arguments
    /// * `instruction` – The jump instruction to emit to the chunk.
    /// * `token` – The token associated with this jump.
    ///
    /// ## Returns
    /// `usize` – The position of the currently emitted jump instruction. This value
    /// should be used by the call to the `patch_jump(...)` function to patch the
    /// correct jump instruction's offset.
    fn emit_jump(&mut self, instruction: bytecode::OpCode, token: &Token) -> usize {
        self.emit_op_code_with_short(instruction, 0xffff, (token.line_num, token.column_num));
        return self.current_chunk_mut().len() - 2;
    }

    /// Patches the offset of a jump instruction.
    ///
    /// ## Arguments
    /// * `offset` – The position in the chunk of the jump instruction to be patched.
    /// * `token` – The token associated with this jump patch.
    fn patch_jump(&mut self, offset: usize, token: &Token) {
        // -2 to adjust for the bytecode for the jump offset itself.
        match u16::try_from(self.current_chunk_mut().len() - offset - 2) {
            Ok(forward) => {
                let jump = forward.to_be_bytes();

                self.current_chunk_mut().modify_byte(offset, jump[0]);
                self.current_chunk_mut().modify_byte(offset + 1, jump[1]);
            }
            Err(_) => {
                return self.error_at_token(
                    token,
                    CompilerErrorType::MaxCapacity,
                    "Too much code to jump over.",
                );
            }
        }
    }

    /// Patches the offset of a jump instruction.
    ///
    /// ## Arguments
    /// * `loop_start` – The position in the chunk of the jump instruction to be patched.
    /// * `token` – The token associated with this jump patch.
    fn emit_loop(&mut self, loop_start: usize, token: &Token) {
        let offset = self.current_chunk_mut().len() - loop_start;
        let line_info = (token.line_num, token.column_num);

        if offset < (u8::MAX - 2) as usize {
            self.emit_op_code(bytecode::OpCode::LoopJump, line_info);

            // +2 to account for the 'OP_LOOP_JUMP' and its operand.
            let jump = (offset + 2) as u8;
            self.emit_raw_byte(jump, line_info);
        } else if offset < (u16::MAX - 3) as usize {
            self.emit_op_code(bytecode::OpCode::LoopJumpLong, line_info);

            // +3 to account for the 'OP_LOOP_JUMP_LONG' and its operands.
            let jump = (offset + 3) as u16;
            self.emit_short(jump, line_info);
        } else {
            return self.error_at_token(token, CompilerErrorType::MaxCapacity, "Loop body too large.");
        }
    }

    /// Emits a constant instruction and adds the related object to the constant pool
    ///
    /// # Arguments
    /// * `obj` – A reference to the literal object being added to the pool.
    /// * `token` – The object's original token.
    /// * `load` – Whether or not we should also emit a LOAD_CONSTANT instruction.
    pub fn add_literal_to_pool(&mut self, obj: Object, token: &Token, load: bool) -> Option<u16> {
        let constant_pos = self.current_chunk_mut().add_constant(obj);
        let opr_pos = (token.line_num, token.column_num);

        match constant_pos {
            Ok(idx) => {
                if load {
                    if idx < 256 {
                        self.emit_op_code(OpCode::LoadConstant, opr_pos);
                        self.emit_raw_byte(idx as u8, opr_pos);
                    } else {
                        self.emit_op_code(OpCode::LoadConstantLong, opr_pos);
                        self.emit_short(idx, opr_pos);
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
    /// ## Arguments
    /// *  `tok` – The token that caused the error.
    /// * `message` – The error message to display.
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
            err_name, token.line_num, token.column_num, message
        );

        self.errors.push(ErrorReport {
            line: token.line_num,
            column: token.column_num,
            lexeme_len: token.lexeme.len(),
            message: msg,
        });
    }
}
