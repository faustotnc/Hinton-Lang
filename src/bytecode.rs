use super::objects::Object;
use num_traits::FromPrimitive;

/// The set of instructions supported by the virtual machine.
///
/// **NOTE:** Changing the order in which members are declared creates
/// incompatibilities between different versions of the interpreter.
#[derive(Debug, PartialEq)]
#[repr(u8)]
#[derive(FromPrimitive)]
pub enum OpCode {
    // Instructions with zero chunk operands.
    // While these instructions do not have any
    // bytecode operands, some of them do have
    // object operands from the stack.
    Add,
    BitwiseAnd,
    BitwiseNot,
    BitwiseOr,
    BitwiseShiftLeft,
    BitwiseShiftRight,
    BitwiseXor,
    Divide,
    Equals,
    Expo,
    ForLoopIterNext,
    GreaterThan,
    GreaterThanEq,
    Indexing,
    LessThan,
    LessThanEq,
    LoadImm0F,
    LoadImm0I,
    LoadImm1F,
    LoadImm1I,
    LoadImmFalse,
    LoadImmNull,
    LoadImmTrue,
    LoadNative,
    LogicNot,
    MakeIter,
    MakeRange,
    Modulus,
    Multiply,
    Negate,
    NotEq,
    NullishCoalescing,
    PopStackTop,
    PopCloseUpVal,
    Subtract,
    EndVirtualMachine,
    Return,

    // Instructions with one chunk operands.
    // These instructions use the next byte
    // from the chunk as its operand.
    BindDefaults,
    FuncCall,
    GetLocal,
    JumpHasNextOrPop,
    LoadConstant,
    LoadImmN,
    LoopJump,
    MakeArray,
    MakeTuple,
    SetLocal,
    DefineGlobal,
    GetGlobal,
    SetGlobal,
    GetUpVal,
    SetUpVal,
    CloseUpVal,

    // Instructions with two chunk operands.
    // These instructions use the next two
    // bytes (a short) as their operands.
    GetLocalLong,
    JumpForward,
    JumpHasNextOrPopLong,
    JumpIfFalseOrPop,
    JumpIfTrueOrPop,
    LoadConstantLong,
    LoadImmNLong,
    LoopJumpLong,
    MakeArrayLong,
    MakeTupleLong,
    PopJumpIfFalse,
    SetLocalLong,
    DefineGlobalLong,
    GetGlobalLong,
    SetGlobalLong,
    GetUpValLong,
    SetUpValLong,
    CloseUpValLong,

    // Instructions with a variable number of instructions.
    MakeClosure,
    // Byte #1 is the position of function object in the pool.
    // --- UpValue Encoding (2 bytes per up_value) ---
    // One byte if up value is local
    // One byte for the position of the up value
    MakeClosureLarge,
    // Byte #1 is the position of function object in the pool.
    // --- UpValue Encoding (3 bytes per up_value) ---
    // One byte if up value is local
    // Two bytes for the position of the up value
    MakeClosureLong,
    // Byte #1 and Byte #2 is the position of function object in the pool.
    // --- UpValue Encoding (2 bytes per up_value) ---
    // One byte if up value is local
    // One byte for the position of the up value
    MakeClosureLongLarge,
    // Byte #1 and Byte #2 is the position of function object in the pool.
    // --- UpValue Encoding (3 bytes per up_value) ---
    // One byte if up value is local
    // Two bytes for the position of the up value
}

/// Contains all the necessary information about
/// the instructions to be executed.
#[derive(Clone)]
pub struct Chunk {
    instructions: Vec<u8>,
    locations: Vec<(usize, usize)>,
    constants: Vec<Object>,
}

impl<'a> Chunk {
    /// Creates a new chunk.
    ///
    /// ## Returns
    /// `Chunk` – a new chunk.
    pub fn new() -> Self {
        Self {
            instructions: vec![],
            locations: Vec::new(),
            constants: Vec::new(),
        }
    }

    /// Adds a constant to this chunk's constants pool
    ///
    /// ## Arguments
    /// * `obj` – The object to be added to the pool
    ///
    /// ## Returns
    /// * `Result<u16, ()>` – If the object was successfully added to the pool,
    /// returns the position of the object in the pool. If the item could not be
    // added because the pool is full, returns error.
    pub fn add_constant(&mut self, obj: Object) -> Result<u16, ()> {
        return if self.constants.len() < (u16::MAX as usize) {
            // Having to create an iterator, then enumerating that iterator, to finally
            // look for the object seems very expensive. Can we do better? Do the benefits
            // of storing a single object in the heap outweigh the cost of these operations?
            // TODO: Find a better way of doing this, or do research to see if the benefits outweigh the cost.
            match self.constants.iter().enumerate().find(|x| x.1.equals(&obj)) {
                Some(x) => Ok(x.0 as u16),
                None => {
                    self.constants.push(obj);
                    Ok((self.constants.len() as u16) - 1)
                }
            }
        } else {
            Err(())
        };
    }

    /// Retrieves a constant from this chunk's constants pool.
    ///
    /// ## Arguments
    /// * `idx` – The index of the object constant.
    ///
    /// ## Returns
    /// `Option<&Object>` – The object at the given index in the constant pool
    pub fn get_constant(&self, idx: usize) -> Option<&Object> {
        self.constants.get(idx)
    }

    /// Returns the OpCode associated with a byte instruction in the
    /// instructions list.
    ///
    /// ## Arguments
    /// * `idx` – The index of the instruction in the instructions list.
    ///
    /// ## Returns
    /// `Option<OpCode>` – The OpCode instruction at the given index.
    pub fn get_op_code(&self, idx: usize) -> Option<OpCode> {
        let byte = self.instructions.get(idx);

        match byte {
            Some(x) => FromPrimitive::from_u8(*x),
            None => None,
        }
    }

    /// Adds a ByteCode instruction from a given OpCode into the
    /// instructions list.
    ///
    /// ## Arguments
    /// * `val` – The OpCode instruction to add to the list.
    pub fn push_op_code(&mut self, val: OpCode) {
        self.instructions.push(val as u8);
    }

    /// Retrieves a raw byte from the instructions list
    ///
    /// ## Arguments
    /// * `idx` – The index of the instruction in the instructions list.
    ///
    /// ## Returns
    /// `Option<u8>` – The byte instruction at the given index.
    pub fn get_byte(&self, idx: usize) -> Option<u8> {
        let byte = self.instructions.get(idx);

        match byte {
            Some(x) => Some(*x),
            None => None,
        }
    }

    /// Adds a raw byte into the instructions list
    ///
    /// ## Arguments
    /// * `val` – The byte to add to the instructions list.
    pub fn push_byte(&mut self, val: u8) {
        self.instructions.push(val);
    }

    /// Retrieves the current byte and the next byte in the instructions list
    /// to form a 16-bit short.
    ///
    /// ## Arguments
    /// * `idx` – The index of the first byte in the instructions list.
    ///
    /// ## Returns
    /// `Option<u16>` – The short generated from the two bytes.
    pub fn get_short(&self, idx: usize) -> Option<u16> {
        let b1 = match self.instructions.get(idx) {
            Some(byte) => byte,
            None => return None,
        };

        let b2 = match self.instructions.get(idx + 1) {
            Some(byte) => byte,
            None => return None,
        };

        return Some(u16::from_be_bytes([*b1, *b2]));
    }

    /// Splits a 16-bit integer into two bytes, and adds each individual
    /// byte in sequence into the instructions list.
    ///
    /// ## Arguments
    /// * `val` – The short to add to the instruction list.
    pub fn push_short(&mut self, val: u16) {
        let short = val.to_be_bytes();

        self.instructions.push(short[0]);
        self.instructions.push(short[1]);
    }

    /// Modifies the byte value at the specified chunk index.
    ///
    /// ## Arguments
    /// * `idx` – The index in the chunk of the byte to be modified.
    /// * `new_val` – The new value of the byte.
    pub fn modify_byte(&mut self, idx: usize, new_val: u8) {
        self.instructions[idx] = new_val;
    }

    pub fn get_line_info(&self, idx: usize) -> Option<&(usize, usize)> {
        return self.locations.get(idx);
    }

    pub fn push_line_info(&mut self, line_info: (usize, usize)) {
        self.locations.push(line_info);
    }

    /// Gets the length of the instructions list.
    ///
    /// ## Returns
    /// * `usize` – The length of the instructions list.
    pub fn len(&self) -> usize {
        return self.instructions.len();
    }

    #[cfg(test)]
    pub fn get_pool_size(&self) -> usize {
        return self.constants.len();
    }
}

/// Disassembles the chunk into its raw bytes, and prints the each instruction byte.
/// This is useful when comparing the chunk generated by a program vs another program.
///
/// ## Arguments
///
/// * `name` – the name to print for the current chunk
#[cfg(feature = "show_raw_bytecode")]
pub fn print_raw(chunk: &Chunk, name: &str) {
    let mut i = 0;

    // prints this chunk's name
    println!("==== {} ====", name);

    while i < chunk.instructions.len() {
        match chunk.get_byte(i) {
            Some(instr) => print!("{:#04X} ", instr as u8),
            None => print!("\x1b[36mNONE\x1b[0m "),
        }

        if (i + 1) % 8 == 0 {
            println!();
        }

        i += 1;
    }

    println!("\n\nChunk Size: {}", i);
    println!("================\n");
}

/// Disassembles the chunk, printing the each instruction and
/// their related information.
///
/// ## Arguments
/// * `name` – the name to print for the current chunk
#[cfg(feature = "show_bytecode")]
pub fn disassemble_function_scope(chunk: &Chunk, name: &String) {
    // prints this chunk's name
    println!("==== {} ====", name);

    let mut current_line = 0;

    let mut idx = 0;
    while idx < chunk.len() {
        let code = chunk.get_byte(idx);
        let line_info = chunk.get_line_info(idx);

        // Prints a line number or a vertical bar indicating that the
        // current instruction is in the same line as the previous one.
        if let Some(location) = line_info {
            if location.0 != current_line {
                print!("{:>05}\t", location.0);
                current_line = location.0;
            } else {
                print!("  |\t")
            }
        }

        // Prints the index of the current instruction
        print!("{:>05} ", idx);

        // Prints the instruction name
        if let Some(instr) = code {
            let op_code_name;
            let mut operand_val = String::from("");

            // Reads two bytes as the index of a constant
            let const_val = |idx: usize, is_long: bool| -> &Object {
                let pos = if is_long {
                    match chunk.get_short(idx) {
                        Some(short) => short as usize,
                        None => unreachable!("Could not get short."),
                    }
                } else {
                    match chunk.get_byte(idx) {
                        Some(byte) => byte as usize,
                        None => unreachable!("Could not get byte."),
                    }
                };

                chunk.get_constant(pos).unwrap()
            };

            // Gets the operand value
            let mut get_operand = |operand_count: usize| {
                idx += operand_count;

                operand_val = if operand_count == 1 {
                    format!("{}", chunk.get_byte(idx).unwrap())
                } else {
                    format!("{}", chunk.get_short(idx - 1).unwrap())
                }
            };

            match FromPrimitive::from_u8(instr).unwrap() {
                OpCode::Add => op_code_name = "ADD",
                OpCode::BitwiseAnd => op_code_name = "BIT_AND",
                OpCode::BitwiseNot => op_code_name = "BIT_NOT",
                OpCode::BitwiseOr => op_code_name = "BIT_OR",
                OpCode::BitwiseShiftLeft => op_code_name = "BIT_SHIFT_L",
                OpCode::BitwiseShiftRight => op_code_name = "BIT_SHIFT_R",
                OpCode::BitwiseXor => op_code_name = "BIT_XOR",
                OpCode::Divide => op_code_name = "DIVIDE",
                OpCode::Equals => op_code_name = "EQUALS",
                OpCode::Expo => op_code_name = "EXPO",
                OpCode::ForLoopIterNext => op_code_name = "FOR_LOOP_ITER_NEXT",
                OpCode::GreaterThan => op_code_name = "GREATER_THAN",
                OpCode::GreaterThanEq => op_code_name = "GREATER_THAN_EQ",
                OpCode::Indexing => op_code_name = "INDEXING",
                OpCode::LessThan => op_code_name = "LESS_THAN",
                OpCode::LessThanEq => op_code_name = "LESS_THAN_EQ",
                OpCode::LoadImm0F => op_code_name = "LOAD_IMM_0F",
                OpCode::LoadImm0I => op_code_name = "LOAD_IMM_0I",
                OpCode::LoadImm1F => op_code_name = "LOAD_IMM_1F",
                OpCode::LoadImm1I => op_code_name = "LOAD_IMM_1I",
                OpCode::LoadImmFalse => op_code_name = "LOAD_IMM_FALSE",
                OpCode::LoadImmNull => op_code_name = "LOAD_IMM_NULL",
                OpCode::LoadImmTrue => op_code_name = "LOAD_IMM_TRUE",
                OpCode::LoadNative => op_code_name = "LOAD_NATIVE",
                OpCode::LogicNot => op_code_name = "LOGIC_NOT",
                OpCode::MakeIter => op_code_name = "MAKE_ITER",
                OpCode::MakeRange => op_code_name = "MAKE_RANGE",
                OpCode::Modulus => op_code_name = "MODULUS",
                OpCode::Multiply => op_code_name = "MULTIPLY",
                OpCode::Negate => op_code_name = "NEGATE",
                OpCode::NotEq => op_code_name = "NOT_EQ",
                OpCode::NullishCoalescing => op_code_name = "NULLISH",
                OpCode::PopStackTop => op_code_name = "POP_STACK_TOP",
                OpCode::Subtract => op_code_name = "SUBTRACT",
                OpCode::PopCloseUpVal => op_code_name = "POP_CLOSE_UP_VAL",
                OpCode::EndVirtualMachine => op_code_name = "END_VIRTUAL_MACHINE",
                OpCode::Return => op_code_name = "RETURN",

                // OpCodes with 1 operand
                OpCode::BindDefaults => {
                    op_code_name = "BIND_DEFAULTS";
                    get_operand(1);
                }
                OpCode::FuncCall => {
                    op_code_name = "FUNC_CALL";
                    get_operand(1);
                }
                OpCode::GetLocal => {
                    op_code_name = "GET_LOCAL";
                    get_operand(1);
                }
                OpCode::JumpHasNextOrPop => {
                    op_code_name = "JUMP_HAS_NEXT_OR_POP";
                    idx += 1;
                    // `idx + 1` because at runtime, the IP points to the next instruction
                    operand_val = format!("{}", (idx + 1) - chunk.get_byte(idx).unwrap() as usize);
                    operand_val += &format!(" (sub {} from IP)", chunk.get_byte(idx).unwrap());
                }
                OpCode::LoadConstant => {
                    op_code_name = "LOAD_CONSTANT";
                    get_operand(1);
                    operand_val += &format!(" -> ({})", const_val(idx, false));
                }
                OpCode::DefineGlobal => {
                    op_code_name = "DEFINE_GLOBAL";
                    get_operand(1);
                    operand_val += &format!(" -> '{}'", const_val(idx, false));
                }
                OpCode::GetGlobal => {
                    op_code_name = "GET_GLOBAL";
                    get_operand(1);
                    operand_val += &format!(" -> '{}'", const_val(idx, false));
                }
                OpCode::SetGlobal => {
                    op_code_name = "SET_GLOBAL";
                    get_operand(1);
                    operand_val += &format!(" -> '{}'", const_val(idx, false));
                }
                OpCode::LoadImmN => {
                    op_code_name = "LOAD_IMM_N";
                    get_operand(1);
                }
                OpCode::LoopJump => {
                    op_code_name = "LOOP_JUMP";
                    idx += 1;
                    // `idx + 1` because at runtime, the IP points to the next instruction
                    operand_val = format!("{}", (idx + 1) - chunk.get_byte(idx).unwrap() as usize);
                    operand_val += &format!(" (sub {} from IP)", chunk.get_byte(idx).unwrap());
                }
                OpCode::MakeArray => {
                    op_code_name = "MAKE_ARRAY";
                    get_operand(1);
                }
                OpCode::MakeTuple => {
                    op_code_name = "MAKE_TUPLE";
                    get_operand(1);
                }
                OpCode::SetLocal => {
                    op_code_name = "SET_LOCAL";
                    get_operand(1);
                }
                OpCode::GetUpVal => {
                    op_code_name = "GET_UP_VAL";
                    get_operand(1);
                }
                OpCode::SetUpVal => {
                    op_code_name = "SET_UP_VAL";
                    get_operand(1);
                }
                OpCode::CloseUpVal => {
                    op_code_name = "CLOSE_UP_VAL";
                    get_operand(1);
                }

                // OpCode with 2 operands
                OpCode::GetLocalLong => {
                    op_code_name = "GET_LOCAL_LONG";
                    get_operand(2);
                }
                OpCode::JumpForward => {
                    op_code_name = "JUMP_FORWARD";
                    idx += 2;
                    let offset = chunk.get_short(idx - 1).unwrap() as usize;
                    // `idx + 1` because at runtime, the IP points to the next instruction
                    operand_val = format!("{} (add {} to IP)", (idx + 1) + offset, offset);
                }
                OpCode::JumpIfFalseOrPop => {
                    op_code_name = "JUMP_IF_FALSE_OR_POP";
                    get_operand(2);
                }
                OpCode::JumpHasNextOrPopLong => {
                    op_code_name = "JUMP_HAS_NEXT_OR_POP_LONG";
                    idx += 2;
                    let offset = chunk.get_short(idx - 1).unwrap() as usize;
                    // `idx + 1` because at runtime, the IP points to the next instruction
                    operand_val = format!("{} (sub {} from IP)", (idx + 1) - offset, offset);
                }
                OpCode::JumpIfTrueOrPop => {
                    op_code_name = "JUMP_IF_TRUE_OR_POP";
                    get_operand(2);
                }
                OpCode::LoadConstantLong => {
                    op_code_name = "LOAD_CONSTANT_LONG";
                    get_operand(2);
                    operand_val += &format!(" -> ({})", const_val(idx - 1, true));
                }
                OpCode::DefineGlobalLong => {
                    op_code_name = "DEFINE_GLOBAL_LONG";
                    get_operand(2);
                    operand_val += &format!(" -> '{}'", const_val(idx - 1, true));
                }
                OpCode::GetGlobalLong => {
                    op_code_name = "GET_GLOBAL_LONG";
                    get_operand(2);
                    operand_val += &format!(" -> '{}'", const_val(idx - 1, true));
                }
                OpCode::SetGlobalLong => {
                    op_code_name = "GET_GLOBAL_LONG";
                    get_operand(2);
                    operand_val += &format!(" -> '{}'", const_val(idx - 1, true));
                }
                OpCode::LoadImmNLong => {
                    op_code_name = "LOAD_IMM_N_LONG";
                    get_operand(2);
                }
                OpCode::LoopJumpLong => {
                    op_code_name = "LOOP_JUMP_LONG";
                    idx += 2;
                    let offset = chunk.get_short(idx - 1).unwrap() as usize;
                    // `idx + 1` because at runtime, the IP points to the next instruction
                    operand_val = format!("{} (sub {} from IP)", (idx + 1) - offset, offset);
                }
                OpCode::MakeArrayLong => {
                    op_code_name = "MAKE_ARRAY_LONG";
                    get_operand(2);
                }
                OpCode::MakeTupleLong => {
                    op_code_name = "MAKE_TUPLE_LONG";
                    get_operand(2);
                }
                OpCode::PopJumpIfFalse => {
                    op_code_name = "POP_JUMP_IF_FALSE";
                    idx += 2;
                    let offset = chunk.get_short(idx - 1).unwrap() as usize;
                    // `idx + 1` because at runtime, the IP points to the next instruction
                    operand_val = format!("{} (add {} to IP)", (idx + 1) + offset, offset);
                }
                OpCode::SetLocalLong => {
                    op_code_name = "SET_LOCAL_LONG";
                    get_operand(2);
                }
                OpCode::GetUpValLong => {
                    op_code_name = "GET_UP_VAL_LONG";
                    get_operand(2);
                }
                OpCode::SetUpValLong => {
                    op_code_name = "SET_UP_VAL_LONG";
                    get_operand(2);
                }
                OpCode::CloseUpValLong => {
                    op_code_name = "CLOSE_UP_VAL_LONG";
                    get_operand(2);
                }

                OpCode::MakeClosure | OpCode::MakeClosureLong => {
                    let up_value_count;
                    if let OpCode::MakeClosure = FromPrimitive::from_u8(instr).unwrap() {
                        op_code_name = "MAKE_CLOSURE";
                        get_operand(1);

                        let obj = const_val(idx, false);
                        up_value_count = obj.as_function().unwrap().up_val_count;
                        operand_val += &format!(" -> '{}'", obj);
                    } else {
                        op_code_name = "MAKE_CLOSURE_LONG";
                        get_operand(2);

                        let obj = const_val(idx, true);
                        up_value_count = obj.as_function().unwrap().up_val_count;
                        operand_val += &format!(" -> '{}'", obj);
                    }

                    for i in 0..up_value_count {
                        if i <= (up_value_count - 1) {
                            operand_val += "\n";
                        }

                        operand_val += &format!("  |\t{:>05}      | {}", idx + 1, i);

                        let is_local = chunk.get_byte(idx + 1).unwrap() == 1u8;
                        let index = chunk.get_byte(idx + 2).unwrap();
                        let up_val_type = if is_local { "Local" } else { "UpVal" };
                        operand_val += &format!(" {} idx={}", up_val_type, index);

                        idx += 2;
                    }
                }

                OpCode::MakeClosureLarge | OpCode::MakeClosureLongLarge => {
                    let up_value_count;
                    if let OpCode::MakeClosureLarge = FromPrimitive::from_u8(instr).unwrap() {
                        op_code_name = "MAKE_CLOSURE_LARGE";
                        get_operand(1);

                        let obj = const_val(idx, false);
                        up_value_count = obj.as_function().unwrap().up_val_count;
                        operand_val += &format!(" -> '{}'", obj);
                    } else {
                        op_code_name = "MAKE_CLOSURE_LONG_LARGE";
                        get_operand(2);

                        let obj = const_val(idx, true);
                        up_value_count = obj.as_function().unwrap().up_val_count;
                        operand_val += &format!(" -> '{}'", obj);
                    }

                    for i in 0..up_value_count {
                        if i <= (up_value_count - 1) {
                            operand_val += "\n";
                        }

                        operand_val += &format!("  |\t{:>05}      | {}", idx + 1, i);

                        let is_local = chunk.get_byte(idx + 1).unwrap() == 1u8;
                        let index = chunk.get_short(idx + 2).unwrap();
                        let up_val_type = if is_local { "Local" } else { "UpVal" };
                        operand_val += &format!(" {} idx={}", up_val_type, index);

                        idx += 3;
                    }
                }
            }

            // Prints the instruction code and instruction name
            println!(
                "\x1b[32m{:#04X}\x1b[0m – \x1b[36m{:<26}\x1b[0m {}",
                instr, op_code_name, operand_val
            );
        } else {
            println!("No Instruction Found...");
        }

        idx += 1;
    }

    println!();
}
