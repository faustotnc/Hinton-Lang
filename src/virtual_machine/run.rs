use crate::ast::{BinaryExprType, UnaryExprType};
use crate::bytecode::OpCode;
use crate::errors::RuntimeErrorType;
use crate::natives::{get_next_in_iter, make_iter};
use crate::objects::{ClassObject, ClosureObject, Object, RangeObject, UpValRef};
use crate::virtual_machine::{RuntimeResult, VirtualMachine};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

impl<'a> VirtualMachine {
   /// Executes the instructions in a chunk.
   pub(crate) fn run(&mut self) -> RuntimeResult {
      loop {
         let instruction = self.get_next_op_code();

         let exec = match instruction {
            OpCode::PopStackTop => {
               self.pop_stack();
               RuntimeResult::Continue
            }

            // Object loaders
            OpCode::LoadConstant | OpCode::LoadConstantLong => self.op_load_constant(),
            OpCode::LoadImm0F => self.push_stack(Object::Float(0f64)),
            OpCode::LoadImm0I => self.push_stack(Object::Int(0i64)),
            OpCode::LoadImm1F => self.push_stack(Object::Float(1f64)),
            OpCode::LoadImm1I => self.push_stack(Object::Int(1i64)),
            OpCode::LoadImmFalse => self.push_stack(Object::Bool(false)),
            OpCode::LoadImmN | OpCode::LoadImmNLong => self.op_load_immediate_n(),
            OpCode::LoadImmNull => self.push_stack(Object::Null),
            OpCode::LoadImmTrue => self.push_stack(Object::Bool(true)),
            OpCode::LoadNative => self.op_load_native(),

            // Object makers
            OpCode::MakeArray | OpCode::MakeArrayLong => self.op_make_array(),
            OpCode::MakeClass | OpCode::MakeClassLong => self.op_make_class(),
            OpCode::MakeClosure | OpCode::MakeClosureLarge => self.op_make_closure(),
            OpCode::MakeClosureLong | OpCode::MakeClosureLongLarge => self.op_make_closure_long(),
            OpCode::MakeDict | OpCode::MakeDictLong => self.op_make_dictionary(),
            OpCode::MakeInstance => self.op_make_instance(),
            OpCode::MakeIter => self.op_make_iter(),
            OpCode::MakeRange => self.op_make_range(),
            OpCode::MakeTuple | OpCode::MakeTupleLong => self.op_make_tuple(),

            // Global declarations
            OpCode::DefineGlobal | OpCode::DefineGlobalLong => self.op_define_global(),
            OpCode::GetGlobal | OpCode::GetGlobalLong => self.op_get_global(),
            OpCode::SetGlobal | OpCode::SetGlobalLong => self.op_set_global(),

            // Local declarations
            OpCode::GetLocal | OpCode::GetLocalLong => self.op_get_local(),
            OpCode::SetLocal | OpCode::SetLocalLong => self.op_set_local(),

            // Operators
            OpCode::Add => self.binary_operation(BinaryExprType::Addition),
            OpCode::BitwiseAnd => self.binary_operation(BinaryExprType::BitwiseAND),
            OpCode::BitwiseNot => self.unary_operation(UnaryExprType::BitwiseNeg),
            OpCode::BitwiseOr => self.binary_operation(BinaryExprType::BitwiseOR),
            OpCode::BitwiseShiftLeft => self.binary_operation(BinaryExprType::BitwiseShiftLeft),
            OpCode::BitwiseShiftRight => self.binary_operation(BinaryExprType::BitwiseShiftRight),
            OpCode::BitwiseXor => self.binary_operation(BinaryExprType::BitwiseXOR),
            OpCode::Divide => self.binary_operation(BinaryExprType::Division),
            OpCode::Equals => self.binary_operation(BinaryExprType::LogicEQ),
            OpCode::Expo => self.binary_operation(BinaryExprType::Expo),
            OpCode::GreaterThan => self.binary_operation(BinaryExprType::LogicGreaterThan),
            OpCode::GreaterThanEq => self.binary_operation(BinaryExprType::LogicGreaterThanEQ),
            OpCode::Indexing => self.op_indexing(),
            OpCode::LessThan => self.binary_operation(BinaryExprType::LogicLessThan),
            OpCode::LessThanEq => self.binary_operation(BinaryExprType::LogicLessThanEQ),
            OpCode::LogicNot => self.unary_operation(UnaryExprType::LogicNeg),
            OpCode::Modulus => self.binary_operation(BinaryExprType::Modulus),
            OpCode::Multiply => self.binary_operation(BinaryExprType::Multiplication),
            OpCode::Negate => self.unary_operation(UnaryExprType::ArithmeticNeg),
            OpCode::NotEq => self.binary_operation(BinaryExprType::LogicNotEQ),
            OpCode::NullishCoalescing => self.binary_operation(BinaryExprType::Nullish),
            OpCode::Subtract => self.binary_operation(BinaryExprType::Minus),

            // Jumps
            OpCode::ForIterNextOrJump => self.op_get_iter_next_or_jump(),
            OpCode::JumpForward => self.op_jump_forward(),
            OpCode::JumpIfFalseOrPop => self.op_jump_if_false_or_pop(),
            OpCode::JumpIfTrueOrPop => self.op_jump_if_true_or_pop(),
            OpCode::LoopJump | OpCode::LoopJumpLong => self.op_loop_jump(),
            OpCode::PopJumpIfFalse => self.op_pop_and_jump_if_false(),

            // Functions and Closures
            OpCode::BindDefaults => self.op_bind_function_defaults(),
            OpCode::CloseUpVal | OpCode::CloseUpValLong => self.up_close_up_value(),
            OpCode::FuncCall => self.op_func_call(),
            OpCode::GetUpVal | OpCode::GetUpValLong => self.op_get_up_value(),
            OpCode::PopCloseUpVal => self.op_pop_stack_and_close_up_value(),
            OpCode::Return => self.op_function_return(),
            OpCode::SetUpVal | OpCode::SetUpValLong => self.op_set_up_value(),

            // Property manipulators
            OpCode::GetProp | OpCode::GetPropLong => self.op_get_property(),
            OpCode::SetProp | OpCode::SetPropLong => self.op_set_property(),

            // VM-Specific
            OpCode::EndVirtualMachine => self.op_end_virtual_machine(),
         };

         // Prints the execution of the program.
         // self.print_execution(instruction);

         match exec {
            RuntimeResult::Continue => continue,
            _ => return exec,
         }
      }
   }

   fn op_end_virtual_machine(&mut self) -> RuntimeResult {
      self.pop_stack(); // Remove the main function off the stack
      self.frames.pop();

      return RuntimeResult::EndOK;
   }

   fn op_pop_and_jump_if_false(&mut self) -> RuntimeResult {
      // The POP_JUMP_IF_FALSE instruction always has a short as its operand.
      let offset = self.get_next_short() as usize;

      if self.pop_stack().is_falsey() {
         self.current_frame_mut().ip += offset;
      }

      RuntimeResult::Continue
   }

   fn op_jump_if_false_or_pop(&mut self) -> RuntimeResult {
      // The JUMP_IF_FALSE_OR_POP instruction always has a short as its operand.
      let offset = self.get_next_short() as usize;

      if self.peek_stack(self.stack.len() - 1).is_falsey() {
         self.current_frame_mut().ip += offset;
      } else {
         self.pop_stack();
      }

      RuntimeResult::Continue
   }

   fn op_jump_if_true_or_pop(&mut self) -> RuntimeResult {
      // The JUMP_IF_TRUE_OR_POP instruction always has a short as its operand.
      let offset = self.get_next_short() as usize;

      if !self.peek_stack(self.stack.len() - 1).is_falsey() {
         self.current_frame_mut().ip += offset;
      } else {
         self.pop_stack();
      }

      RuntimeResult::Continue
   }

   fn op_jump_forward(&mut self) -> RuntimeResult {
      // The JUMP_FORWARD instruction always has a short as its operand.
      let offset = self.get_next_short() as usize;
      self.current_frame_mut().ip += offset;
      RuntimeResult::Continue
   }

   fn op_loop_jump(&mut self) -> RuntimeResult {
      let offset = self.get_std_or_long_operand(OpCode::LoopJump);
      self.current_frame_mut().ip -= offset;
      RuntimeResult::Continue
   }

   fn op_load_native(&mut self) -> RuntimeResult {
      let native = self.get_next_byte() as usize;

      match self.natives.get_native_fn_object(native) {
         Ok(f) => self.push_stack(Object::Native(Box::new(f))),
         Err(e) => return e,
      }
   }

   fn op_func_call(&mut self) -> RuntimeResult {
      // Functions can only have 255-MAX parameters
      let arg_count = self.get_next_byte();

      let maybe_function = self
         .peek_stack(self.stack.len() - (arg_count as usize) - 1)
         .clone();

      self.call_object(maybe_function, arg_count)
   }

   fn op_make_closure(&mut self) -> RuntimeResult {
      let pos = self.get_std_or_long_operand(OpCode::MakeClosure);

      let function = match self.read_constant(pos).clone() {
         Object::Function(obj) => obj,
         _ => unreachable!("Expected a function object for closure."),
      };

      let up_val_count = function.borrow().up_val_count;
      let mut up_values: Vec<Rc<RefCell<UpValRef>>> = Vec::with_capacity(up_val_count);

      for _ in 0..up_val_count {
         let is_local = self.get_next_byte() == 1u8;
         let index = self.get_next_byte() as usize;

         let up = if is_local {
            self.create_up_value(self.current_frame().return_index + index)
         } else {
            self.current_frame().closure.up_values[index].clone()
         };

         up_values.push(up.clone());
      }

      self.push_stack(Object::Closure(ClosureObject { function, up_values }))
   }

   fn op_make_closure_long(&mut self) -> RuntimeResult {
      let pos = self.get_std_or_long_operand(OpCode::MakeClosureLong);

      let function = match self.read_constant(pos).clone() {
         Object::Function(obj) => obj,
         _ => unreachable!("Expected a function object for closure."),
      };

      let up_val_count = function.borrow().up_val_count;
      let mut up_values: Vec<Rc<RefCell<UpValRef>>> = Vec::with_capacity(up_val_count);

      for _ in 0..up_val_count {
         let is_local = self.get_next_byte() == 1u8;
         let index = self.get_next_short() as usize;

         let up = if is_local {
            self.create_up_value(self.current_frame().return_index + index)
         } else {
            self.current_frame().closure.up_values[index].clone()
         };

         up_values.push(up.clone());
      }

      self.push_stack(Object::Closure(ClosureObject { function, up_values }))
   }

   fn op_get_up_value(&mut self) -> RuntimeResult {
      let pos = self.get_std_or_long_operand(OpCode::GetUpVal);

      let val = match &*self.get_up_val(pos).borrow() {
         UpValRef::Open(l) => self.peek_stack(*l).clone(),
         UpValRef::Closed(o) => o.clone(),
      };

      self.push_stack(val)
   }

   fn op_set_up_value(&mut self) -> RuntimeResult {
      let pos = self.get_std_or_long_operand(OpCode::SetUpVal);
      let new_val = self.stack.last().unwrap().clone();

      match &mut *self.get_up_val(pos).borrow_mut() {
         UpValRef::Open(l) => self.stack[*l] = new_val,
         UpValRef::Closed(u) => *u = new_val,
      }

      RuntimeResult::Continue
   }

   fn up_close_up_value(&mut self) -> RuntimeResult {
      let pos = self.get_std_or_long_operand(OpCode::CloseUpVal);

      for u in self.up_values.iter() {
         if u.borrow().is_open_at(self.current_frame().return_index + pos) {
            let new_val = self.peek_stack(self.current_frame().return_index + pos);
            u.replace(UpValRef::Closed(new_val.clone()));
            break;
         }
      }

      RuntimeResult::Continue
   }

   fn op_pop_stack_and_close_up_value(&mut self) -> RuntimeResult {
      let new_val = self.pop_stack();

      for u in self.up_values.iter() {
         if u.borrow().is_open_at(self.stack.len()) {
            u.replace(UpValRef::Closed(new_val));
            break;
         }
      }

      RuntimeResult::Continue
   }

   fn op_bind_function_defaults(&mut self) -> RuntimeResult {
      // Functions can only have 255-MAX parameters
      let param_count = self.get_next_byte();

      let mut defaults: Vec<Object> = vec![];
      for _ in 0..param_count {
         let p = self.pop_stack();
         defaults.push(p);
      }
      defaults.reverse();

      match self.peek_stack_mut(self.stack.len() - 1) {
         Object::Function(m) => {
            m.borrow_mut().defaults = defaults;
         }
         Object::Closure(m) => {
            m.function.borrow_mut().defaults = defaults;
         }
         _ => unreachable!("Expected a function object on TOS."),
      }

      RuntimeResult::Continue
   }

   fn op_function_return(&mut self) -> RuntimeResult {
      let result = self.pop_stack();
      let locals_to_pop = self.stack.len() - self.current_frame().return_index;

      // Pops local declarations from the stack
      for _ in 0..locals_to_pop {
         self.pop_stack();
      }

      // removes the call frame
      self.frames.pop();
      self.push_stack(result)
   }

   fn op_make_class(&mut self) -> RuntimeResult {
      let pos = self.get_std_or_long_operand(OpCode::MakeClass);

      let name = match self.read_constant(pos).clone() {
         Object::String(s) => s,
         _ => unreachable!("Expected string for class name."),
      };

      let new_class = Object::Class(ClassObject { name });
      self.push_stack(new_class)
   }

   fn op_make_instance(&mut self) -> RuntimeResult {
      // Instances can only have 255-MAX arguments
      let arg_count = self.get_next_byte();

      let maybe_instance = self
         .peek_stack(self.stack.len() - (arg_count as usize) - 1)
         .clone();

      self.create_instance(maybe_instance, arg_count)
   }

   fn op_get_property(&mut self) -> RuntimeResult {
      let pos = self.get_std_or_long_operand(OpCode::GetProp);

      let prop_name = match self.read_constant(pos).clone() {
         Object::String(name) => name,
         _ => unreachable!("Expected string for 'GetProp' name."),
      };

      match self.pop_stack() {
         Object::Instance(x) => {
            if x.borrow().fields.contains_key(&prop_name) {
               let val = x.borrow().fields.get(&prop_name).unwrap().clone();
               self.push_stack(val)
            } else {
               return RuntimeResult::Error {
                  error: RuntimeErrorType::ReferenceError,
                  message: format!(
                     "Property '{}' not defined for objects of type '{}'.",
                     prop_name,
                     x.borrow().class.name
                  ),
               };
            }
         }
         Object::Dict(x) => {
            if x.borrow().contains_key(&prop_name) {
               let val = x.borrow().get(&prop_name).unwrap().clone();
               self.push_stack(val)
            } else {
               return RuntimeResult::Error {
                  error: RuntimeErrorType::ReferenceError,
                  message: format!("Entry with key '{}' not found in the dictionary.", prop_name),
               };
            }
         }
         _ => todo!("Other objects also have properties."),
      }
   }

   fn op_set_property(&mut self) -> RuntimeResult {
      let pos = self.get_std_or_long_operand(OpCode::SetProp);

      let prop_name = match self.read_constant(pos).clone() {
         Object::String(name) => name,
         _ => unreachable!("Expected string for 'SetProp' name."),
      };

      let value = self.pop_stack();

      match self.pop_stack() {
         Object::Instance(x) => {
            x.borrow_mut().fields.insert(prop_name, value.clone());
            self.push_stack(value)
         }
         _ => todo!("Other objects also have properties."),
      }
   }

   fn op_make_range(&mut self) -> RuntimeResult {
      let right = self.pop_stack();
      let left = self.pop_stack();

      if left.is_int() && right.is_int() {
         let a = left.as_int().unwrap();
         let b = right.as_int().unwrap();
         self.push_stack(Object::Range(RangeObject { min: a, max: b }))
      } else {
         return RuntimeResult::Error {
            error: RuntimeErrorType::TypeError,
            message: format!(
               "Operation '..' not defined for operands of type '{}' and '{}'.",
               left.type_name(),
               right.type_name()
            ),
         };
      }
   }

   fn unary_operation(&mut self, opr: UnaryExprType) -> RuntimeResult {
      let val = self.pop_stack();

      let result = match opr {
         UnaryExprType::ArithmeticNeg => -val,
         UnaryExprType::LogicNeg => Ok(Object::Bool(val.is_falsey())),
         UnaryExprType::BitwiseNeg => !val,
      };

      match result {
         Ok(r) => self.push_stack(r),
         Err(e) => return e.to_runtime_error(),
      }
   }

   fn binary_operation(&mut self, opr: BinaryExprType) -> RuntimeResult {
      let val2 = self.pop_stack();
      let val1 = self.pop_stack();

      let result = match opr {
         BinaryExprType::Addition => val1 + val2,
         BinaryExprType::BitwiseAND => val1 & val2,
         BinaryExprType::BitwiseOR => val1 | val2,
         BinaryExprType::BitwiseShiftLeft => val1 << val2,
         BinaryExprType::BitwiseShiftRight => val1 >> val2,
         BinaryExprType::BitwiseXOR => val1 ^ val2,
         BinaryExprType::Division => val1 / val2,
         BinaryExprType::Expo => val1.pow(val2),
         BinaryExprType::LogicGreaterThan => val1.gt(val2),
         BinaryExprType::LogicGreaterThanEQ => val1.gteq(val2),
         BinaryExprType::LogicLessThan => val1.lt(val2),
         BinaryExprType::LogicLessThanEQ => val1.lteq(val2),
         BinaryExprType::Minus => val1 - val2,
         BinaryExprType::Modulus => val1 % val2,
         BinaryExprType::Multiplication => val1 * val2,
         BinaryExprType::LogicEQ => Ok(Object::Bool(val1.equals(&val2))),
         BinaryExprType::LogicNotEQ => Ok(Object::Bool(!val1.equals(&val2))),
         BinaryExprType::Nullish => {
            if val1.is_null() {
               Ok(val2)
            } else {
               Ok(val1)
            }
         }
         _ => unreachable!("The other binary operations have special instruction methods."),
      };

      match result {
         Ok(r) => self.push_stack(r),
         Err(e) => return e.to_runtime_error(),
      }
   }

   fn op_get_iter_next_or_jump(&mut self) -> RuntimeResult {
      let jump = self.get_next_short() as usize;

      match self.peek_stack(self.stack.len() - 1) {
         Object::Iter(i) => match get_next_in_iter(i) {
            Ok(o) => self.push_stack(o),
            Err(_) => {
               self.pop_stack();
               self.current_frame_mut().ip += jump;
               RuntimeResult::Continue
            }
         },
         _ => unreachable!("Expected iterable object on TOS."),
      }
   }

   fn op_make_array(&mut self) -> RuntimeResult {
      // The number of values to pop from the stack. Essentially the size of the array.
      let size = self.get_std_or_long_operand(OpCode::MakeArray);
      let mut arr_values: Vec<Object> = vec![];

      for _ in 0..size {
         arr_values.push(self.pop_stack());
      }

      let arr = Rc::new(RefCell::new(arr_values));
      self.push_stack(Object::Array(arr))
   }

   fn op_make_tuple(&mut self) -> RuntimeResult {
      // The number of values to pop from the stack. Essentially the size of the array.
      let size = self.get_std_or_long_operand(OpCode::MakeTuple);
      let mut tuple_values: Vec<Object> = Vec::with_capacity(size);

      for _ in 0..size {
         tuple_values.push(self.pop_stack());
      }

      let tup = Box::new(tuple_values);
      self.push_stack(Object::Tuple(tup))
   }

   fn op_make_dictionary(&mut self) -> RuntimeResult {
      // The number of values to pop from the stack. Essentially the size of the array.
      let size = self.get_std_or_long_operand(OpCode::MakeDict);
      let mut dict: HashMap<String, Object> = HashMap::new();

      for _ in 0..size {
         let value = self.pop_stack();

         match self.pop_stack() {
            Object::String(key) => {
               dict.insert(key, value);
            }
            _ => unreachable!("Expected string for dictionary key."),
         }
      }

      self.push_stack(Object::Dict(Rc::new(RefCell::new(dict))))
   }

   fn op_indexing(&mut self) -> RuntimeResult {
      let index = self.pop_stack();
      let target = self.pop_stack();

      match target.get_at_index(&index) {
         Ok(r) => self.push_stack(r),
         Err(e) => return e.to_runtime_error(),
      }
   }

   fn op_get_local(&mut self) -> RuntimeResult {
      // The position of the local variable's value in the stack
      let pos = self.get_std_or_long_operand(OpCode::GetLocal);

      let idx = self.current_frame().return_index + pos;
      let value = self.peek_stack(idx).clone();
      self.push_stack(value)
   }

   fn op_set_local(&mut self) -> RuntimeResult {
      // The position of the local variable's value in the stack
      let pos = self.get_std_or_long_operand(OpCode::SetLocal);

      let value = self.stack.last().unwrap();
      let offset = self.current_frame().return_index;

      self.stack[pos + offset] = value.clone();
      RuntimeResult::Continue
   }

   fn op_define_global(&mut self) -> RuntimeResult {
      let pos = self.get_std_or_long_operand(OpCode::DefineGlobal);

      // Gets the name from the pool assigns the value to the global
      if let Object::String(name) = self.read_constant(pos).clone() {
         let val = self.pop_stack();
         self.globals.insert(name, val);
         RuntimeResult::Continue
      } else {
         unreachable!("Expected a string for global declaration name.");
      }
   }

   fn op_get_global(&mut self) -> RuntimeResult {
      let pos = self.get_std_or_long_operand(OpCode::GetGlobal);

      // Gets the name from the pool
      if let Object::String(name) = self.read_constant(pos).clone() {
         let val = self.globals.get(&name).unwrap().clone();
         self.push_stack(val)
      } else {
         unreachable!("Expected a string as global declaration name.");
      }
   }

   fn op_set_global(&mut self) -> RuntimeResult {
      let pos = self.get_std_or_long_operand(OpCode::SetGlobal);

      // Gets the name from the pool
      if let Object::String(name) = self.read_constant(pos).clone() {
         let val = self.stack.last().unwrap().clone();
         self.globals.insert(name, val);
         RuntimeResult::Continue
      } else {
         unreachable!("Expected a string as global declaration name.");
      }
   }

   fn op_make_iter(&mut self) -> RuntimeResult {
      let tos = self.pop_stack();

      match make_iter(tos) {
         Ok(iter) => self.push_stack(iter),
         Err(e) => return e,
      }
   }

   fn op_load_immediate_n(&mut self) -> RuntimeResult {
      let imm = self.get_std_or_long_operand(OpCode::LoadImmN) as i64;
      self.push_stack(Object::Int(imm))
   }

   fn op_load_constant(&mut self) -> RuntimeResult {
      let pos = self.get_std_or_long_operand(OpCode::LoadConstant);
      let val = self.read_constant(pos).clone();
      self.push_stack(val)
   }
}
