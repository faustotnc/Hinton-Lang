#[cfg(feature = "show_bytecode")]
use crate::core::chunk::Chunk;
#[cfg(feature = "show_bytecode")]
use crate::objects::Object;
#[cfg(feature = "show_bytecode")]
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
   // Although these instructions do not have any bytecode operands, some of them do have object
   // operands from the stack.
   Add,
   AppendConstField,
   AppendMethod,
   AppendVarField,
   BitwiseAnd,
   BitwiseNot,
   BitwiseOr,
   BitwiseShiftLeft,
   BitwiseShiftRight,
   BitwiseXor,
   Divide,
   EndVirtualMachine,
   Equals,
   Expo,
   GreaterThan,
   GreaterThanEq,
   LessThan,
   LessThanEq,
   LoadImm0F,
   LoadImm0I,
   LoadImm1F,
   LoadImm1I,
   LoadImmFalse,
   LoadImmNull,
   LoadImmTrue,
   LogicNot,
   MakeIter,
   MakeRange,
   Modulus,
   Multiply,
   Negate,
   NotEq,
   NullishCoalescing,
   PopCloseUpVal,
   PopStackTop,
   Return,
   Subscript,
   SubscriptAssign,
   Subtract,

   // Instructions with one chunk operands.
   // These instructions use the next byte
   // from the chunk as its operand.
   BindDefaults,
   CloseUpVal,
   DefineGlobal,
   FuncCall,
   GetGlobal,
   GetLocal,
   GetProp,
   GetUpVal,
   LoadConstant,
   LoadImmN,
   LoadNative,
   LoadPrimitive,
   LoopJump,
   MakeArray,
   MakeClass,
   MakeDict,
   MakeInstance,
   MakeTuple,
   SetGlobal,
   SetLocal,
   SetProp,
   SetUpVal,

   // Instructions with two chunk operands.
   // These instructions use the next two
   // bytes (a short) as their operands.
   CloseUpValLong,
   DefineGlobalLong,
   ForIterNextOrJump,
   GetGlobalLong,
   GetLocalLong,
   GetPropLong,
   GetUpValLong,
   JumpForward,
   JumpIfFalseOrPop,
   JumpIfTrueOrPop,
   LoadConstantLong,
   LoadImmNLong,
   LoopJumpLong,
   MakeArrayLong,
   MakeClassLong,
   MakeDictLong,
   MakeTupleLong,
   PopJumpIfFalse,
   SetGlobalLong,
   SetLocalLong,
   SetPropLong,
   SetUpValLong,

   // Instructions with a variable number of instructions.
   MakeClosure,
   // Byte #1 is the position of the function object in the pool.
   // --- UpValue Encoding (2 bytes per up_value) ---
   // One byte if up value is local
   // One byte for the position of the up value
   MakeClosureLong,
   // Byte #1 and Byte #2 are the position of the function object in the pool.
   // --- UpValue Encoding (2 bytes per up_value) ---
   // One byte if up value is local
   // One byte for the position of the up value
   MakeClosureLarge,
   // Byte #1 is the position of the function object in the pool.
   // --- UpValue Encoding (3 bytes per up_value) ---
   // One byte if up value is local
   // Two bytes for the position of the up value
   MakeClosureLongLarge,
   // Byte #1 and Byte #2 are the position of the function object in the pool.
   // --- UpValue Encoding (3 bytes per up_value) ---
   // One byte if up value is local
   // Two bytes for the position of the up value
}

/// Disassembles the chunk into its raw bytes, and prints each instruction byte.
#[cfg(feature = "show_raw_bytecode")]
pub fn print_raw(chunk: &Chunk, name: &str) {
   let mut i = 0;

   // prints this chunk's name
   println!("==== {} ====", name);

   while i < chunk.len() {
      let instr = chunk.get_byte(i);
      print!("{:#04X} ", instr as u8);

      if (i + 1) % 8 == 0 {
         println!();
      }

      i += 1;
   }

   println!("\n\nChunk Size: {}", i);
   println!("================\n");
}

/// Disassembles the chunk, printing each instruction, and its related information.
#[cfg(feature = "show_bytecode")]
pub fn disassemble_function_scope(
   chunk: &Chunk,
   natives: &Vec<String>,
   primitives: &Vec<String>,
   name: &String,
) {
   // prints this chunk's name
   println!("==== {} ====", name);

   let mut current_line = 0;

   let mut idx = 0;
   while idx < chunk.len() {
      let code = chunk.get_byte(idx);
      let line_info = chunk.get_line_info(idx);

      // Prints a line number or a vertical bar indicating that the
      // current instruction is in the same line as the previous one.
      if line_info.0 != current_line {
         print!("{:>05}\t", line_info.0);
         current_line = line_info.0;
      } else {
         print!("  |\t")
      }

      // Prints the index of the current instruction
      print!("{:>05} ", idx);

      // Prints the instruction name
      let mut operand_val = String::from("");

      // Reads two bytes as the index of a constant
      let const_val = |idx: usize, is_long: bool| -> &Object {
         let pos = if is_long {
            chunk.get_short(idx) as usize
         } else {
            chunk.get_byte(idx) as usize
         };

         chunk.get_constant(pos)
      };

      // Gets the operand value
      let mut get_operand = |operand_count: usize| {
         idx += operand_count;

         operand_val = if operand_count == 1 {
            format!("{}", chunk.get_byte(idx))
         } else {
            format!("{}", chunk.get_short(idx - 1))
         }
      };

      let op_code_name = match FromPrimitive::from_u8(code).unwrap() {
         OpCode::Add => "ADD",
         OpCode::AppendConstField => "APPEND_CONST_FIELD",
         OpCode::AppendMethod => "APPEND_METHOD",
         OpCode::AppendVarField => "APPEND_VAR_FIELD",
         OpCode::BitwiseAnd => "BIT_AND",
         OpCode::BitwiseNot => "BIT_NOT",
         OpCode::BitwiseOr => "BIT_OR",
         OpCode::BitwiseShiftLeft => "BIT_SHIFT_L",
         OpCode::BitwiseShiftRight => "BIT_SHIFT_R",
         OpCode::BitwiseXor => "BIT_XOR",
         OpCode::Divide => "DIVIDE",
         OpCode::EndVirtualMachine => "END_VIRTUAL_MACHINE",
         OpCode::Equals => "EQUALS",
         OpCode::Expo => "EXPO",
         OpCode::GreaterThan => "GREATER_THAN",
         OpCode::GreaterThanEq => "GREATER_THAN_EQ",
         OpCode::LessThan => "LESS_THAN",
         OpCode::LessThanEq => "LESS_THAN_EQ",
         OpCode::LoadImm0F => "LOAD_IMM_0F",
         OpCode::LoadImm0I => "LOAD_IMM_0I",
         OpCode::LoadImm1F => "LOAD_IMM_1F",
         OpCode::LoadImm1I => "LOAD_IMM_1I",
         OpCode::LoadImmFalse => "LOAD_IMM_FALSE",
         OpCode::LoadImmNull => "LOAD_IMM_NULL",
         OpCode::LoadImmTrue => "LOAD_IMM_TRUE",
         OpCode::LogicNot => "LOGIC_NOT",
         OpCode::MakeIter => "MAKE_ITER",
         OpCode::MakeRange => "MAKE_RANGE",
         OpCode::Modulus => "MODULUS",
         OpCode::Multiply => "MULTIPLY",
         OpCode::Negate => "NEGATE",
         OpCode::NotEq => "NOT_EQ",
         OpCode::NullishCoalescing => "NULLISH",
         OpCode::PopCloseUpVal => "POP_CLOSE_UP_VAL",
         OpCode::PopStackTop => "POP_STACK_TOP",
         OpCode::Return => "RETURN",
         OpCode::Subscript => "SUBSCRIPT",
         OpCode::SubscriptAssign => "SUBSCRIPT_ASSIGN",
         OpCode::Subtract => "SUBTRACT",

         // OpCodes with 1 operand
         OpCode::BindDefaults => {
            get_operand(1);
            "BIND_DEFAULTS"
         }
         OpCode::FuncCall => {
            get_operand(1);
            "FUNC_CALL"
         }
         OpCode::GetLocal => {
            get_operand(1);
            "GET_LOCAL"
         }
         OpCode::LoadConstant => {
            get_operand(1);
            operand_val += &format!(" -> ({})", const_val(idx, false));
            "LOAD_CONSTANT"
         }
         OpCode::DefineGlobal => {
            get_operand(1);
            operand_val += &format!(" -> '{}'", const_val(idx, false));
            "DEFINE_GLOBAL"
         }
         OpCode::GetGlobal => {
            get_operand(1);
            operand_val += &format!(" -> '{}'", const_val(idx, false));
            "GET_GLOBAL"
         }
         OpCode::MakeClass => {
            get_operand(1);
            operand_val += &format!(" -> '{}'", const_val(idx, false));
            "MAKE_CLASS"
         }
         OpCode::MakeInstance => {
            get_operand(1);
            operand_val += &format!(" -> '{}'", const_val(idx, false));
            "MAKE_INSTANCE"
         }
         OpCode::GetProp => {
            get_operand(1);
            operand_val += &format!(" -> '{}'", const_val(idx, false));
            "GET_PROPERTY"
         }
         OpCode::SetProp => {
            get_operand(1);
            operand_val += &format!(" -> '{}'", const_val(idx, false));
            "SET_PROPERTY"
         }
         OpCode::SetGlobal => {
            get_operand(1);
            operand_val += &format!(" -> '{}'", const_val(idx, false));
            "SET_GLOBAL"
         }
         OpCode::LoadImmN => {
            get_operand(1);
            "LOAD_IMM_N"
         }
         OpCode::LoopJump => {
            idx += 1;
            // `idx + 1` because at runtime, the IP points to the next instruction
            operand_val = format!("{}", (idx + 1) - chunk.get_byte(idx) as usize);
            operand_val += &format!(" (sub {} from IP)", chunk.get_byte(idx));
            "LOOP_JUMP"
         }
         OpCode::MakeArray => {
            get_operand(1);
            "MAKE_ARRAY"
         }
         OpCode::MakeTuple => {
            get_operand(1);
            "MAKE_TUPLE"
         }
         OpCode::MakeDict => {
            get_operand(1);
            "MAKE_DICT"
         }
         OpCode::SetLocal => {
            get_operand(1);
            "SET_LOCAL"
         }
         OpCode::GetUpVal => {
            get_operand(1);
            "GET_UP_VAL"
         }
         OpCode::SetUpVal => {
            get_operand(1);
            "SET_UP_VAL"
         }
         OpCode::CloseUpVal => {
            get_operand(1);
            "CLOSE_UP_VAL"
         }
         OpCode::LoadNative => {
            get_operand(1);
            operand_val += &format!(" -> '{}'", natives[chunk.get_byte(idx) as usize]);
            "LOAD_NATIVE"
         }
         OpCode::LoadPrimitive => {
            get_operand(1);
            operand_val += &format!(" -> '{}'", primitives[chunk.get_byte(idx) as usize]);
            "LOAD_PRIMITIVE"
         }

         // OpCode with 2 operands
         OpCode::GetLocalLong => {
            get_operand(2);
            "GET_LOCAL_LONG"
         }
         OpCode::JumpForward => {
            idx += 2;
            let offset = chunk.get_short(idx - 1) as usize;
            // `idx + 1` because at runtime, the IP points to the next instruction
            operand_val = format!("{} (add {} to IP)", (idx + 1) + offset, offset);
            "JUMP_FORWARD"
         }
         OpCode::ForIterNextOrJump => {
            idx += 2;
            let offset = chunk.get_short(idx - 1) as usize;
            // `idx + 1` because at runtime, the IP points to the next instruction
            operand_val = format!("{} (add {} to IP)", (idx + 1) + offset, offset);
            "FOR_ITER_NEXT_OR_JUMP"
         }
         OpCode::JumpIfFalseOrPop => {
            get_operand(2);
            "JUMP_IF_FALSE_OR_POP"
         }
         OpCode::JumpIfTrueOrPop => {
            get_operand(2);
            "JUMP_IF_TRUE_OR_POP"
         }
         OpCode::LoadConstantLong => {
            get_operand(2);
            operand_val += &format!(" -> ({})", const_val(idx - 1, true));
            "LOAD_CONSTANT_LONG"
         }
         OpCode::DefineGlobalLong => {
            get_operand(2);
            operand_val += &format!(" -> '{}'", const_val(idx - 1, true));
            "DEFINE_GLOBAL_LONG"
         }
         OpCode::GetGlobalLong => {
            get_operand(2);
            operand_val += &format!(" -> '{}'", const_val(idx - 1, true));
            "GET_GLOBAL_LONG"
         }
         OpCode::SetGlobalLong => {
            get_operand(2);
            operand_val += &format!(" -> '{}'", const_val(idx - 1, true));
            "GET_GLOBAL_LONG"
         }
         OpCode::LoadImmNLong => {
            get_operand(2);
            "LOAD_IMM_N_LONG"
         }
         OpCode::LoopJumpLong => {
            idx += 2;
            let offset = chunk.get_short(idx - 1) as usize;
            // `idx + 1` because at runtime, the IP points to the next instruction
            operand_val = format!("{} (sub {} from IP)", (idx + 1) - offset, offset);
            "LOOP_JUMP_LONG"
         }
         OpCode::MakeArrayLong => {
            get_operand(2);
            "MAKE_ARRAY_LONG"
         }
         OpCode::MakeTupleLong => {
            get_operand(2);
            "MAKE_TUPLE_LONG"
         }
         OpCode::MakeDictLong => {
            get_operand(2);
            "MAKE_DICT_LONG"
         }
         OpCode::PopJumpIfFalse => {
            idx += 2;
            let offset = chunk.get_short(idx - 1) as usize;
            // `idx + 1` because at runtime, the IP points to the next instruction
            operand_val = format!("{} (add {} to IP)", (idx + 1) + offset, offset);
            "POP_JUMP_IF_FALSE"
         }
         OpCode::SetLocalLong => {
            get_operand(2);
            "SET_LOCAL_LONG"
         }
         OpCode::GetUpValLong => {
            get_operand(2);
            "GET_UP_VAL_LONG"
         }
         OpCode::SetUpValLong => {
            get_operand(2);
            "SET_UP_VAL_LONG"
         }
         OpCode::CloseUpValLong => {
            get_operand(2);
            "CLOSE_UP_VAL_LONG"
         }
         OpCode::MakeClassLong => {
            get_operand(2);
            operand_val += &format!(" -> ({})", const_val(idx - 1, true));
            "MAKE_CLASS_LONG"
         }
         OpCode::GetPropLong => {
            get_operand(2);
            operand_val += &format!(" -> '{}'", const_val(idx, true));
            "GET_PROPERTY_LONG"
         }
         OpCode::SetPropLong => {
            get_operand(2);
            operand_val += &format!(" -> '{}'", const_val(idx, true));
            "SET_PROPERTY_LONG"
         }

         OpCode::MakeClosure | OpCode::MakeClosureLong => {
            let up_value_count;
            let op_name = if let OpCode::MakeClosure = FromPrimitive::from_u8(code).unwrap() {
               get_operand(1);

               let obj = const_val(idx, false);
               up_value_count = obj.as_function().unwrap().borrow().up_val_count;
               operand_val += &format!(" -> '{}'", obj);
               "MAKE_CLOSURE"
            } else {
               get_operand(2);

               let obj = const_val(idx, true);
               up_value_count = obj.as_function().unwrap().borrow().up_val_count;
               operand_val += &format!(" -> '{}'", obj);
               "MAKE_CLOSURE_LONG"
            };

            for i in 0..up_value_count {
               if i <= (up_value_count - 1) {
                  operand_val += "\n";
               }

               operand_val += &format!("  |\t{:>05}      | {}", idx + 1, i);

               let is_local = chunk.get_byte(idx + 1) == 1u8;
               let index = chunk.get_byte(idx + 2);
               let up_val_type = if is_local { "Local" } else { "UpVal" };
               operand_val += &format!(" {} idx={}", up_val_type, index);

               idx += 2;
            }

            op_name
         }

         OpCode::MakeClosureLarge | OpCode::MakeClosureLongLarge => {
            let up_value_count;
            let op_name = if let OpCode::MakeClosureLarge = FromPrimitive::from_u8(code).unwrap() {
               get_operand(1);

               let obj = const_val(idx, false);
               up_value_count = obj.as_function().unwrap().borrow().up_val_count;
               operand_val += &format!(" -> '{}'", obj);
               "MAKE_CLOSURE_LARGE"
            } else {
               get_operand(2);

               let obj = const_val(idx, true);
               up_value_count = obj.as_function().unwrap().borrow().up_val_count;
               operand_val += &format!(" -> '{}'", obj);
               "MAKE_CLOSURE_LONG_LARGE"
            };

            for i in 0..up_value_count {
               if i <= (up_value_count - 1) {
                  operand_val += "\n";
               }

               operand_val += &format!("  |\t{:>05}      | {}", idx + 1, i);

               let is_local = chunk.get_byte(idx + 1) == 1u8;
               let index = chunk.get_short(idx + 2);
               let up_val_type = if is_local { "Local" } else { "UpVal" };
               operand_val += &format!(" {} idx={}", up_val_type, index);

               idx += 3;
            }

            op_name
         }
      };

      // Prints the instruction code and instruction name
      println!(
         "\x1b[32m{:#04X}\x1b[0m – \x1b[36m{:<26}\x1b[0m {}",
         code, op_code_name, operand_val
      );

      idx += 1;
   }

   println!();
}
