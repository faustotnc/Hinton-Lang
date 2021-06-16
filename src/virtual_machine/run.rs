use std::{cell::RefCell, rc::Rc};

use super::{RuntimeErrorType, RuntimeResult, VirtualMachine};
use crate::{
    bytecode::OpCode,
    natives::{get_next_in_iter, iter_has_next, make_iter},
    objects::{ClassObject, ClosureObject, Object, RangeObject, TupleObject, UpValRef},
};

impl<'a> VirtualMachine {
    /// Executes the instruction in a chunk. This is where the interpreter
    /// will spend most its life, therefore, optimizing every function in this file
    /// is very important.
    ///
    /// ## Returns
    /// `InterpretResult` – The result of the execution.
    pub(crate) fn run(&mut self) -> RuntimeResult {
        loop {
            let instruction = self.get_next_op_code();

            match instruction {
                OpCode::LoadImmNull => self.push_stack(Object::Null),
                OpCode::LoadImmTrue => self.push_stack(Object::Bool(true)),
                OpCode::LoadImmFalse => self.push_stack(Object::Bool(false)),
                OpCode::LoadImm0F => self.push_stack(Object::Float(0f64)),
                OpCode::LoadImm0I => self.push_stack(Object::Int(0i64)),
                OpCode::LoadImm1F => self.push_stack(Object::Float(1f64)),
                OpCode::LoadImm1I => self.push_stack(Object::Int(1i64)),

                OpCode::PopStackTop => {
                    self.pop_stack();
                }

                OpCode::LoadImmN | OpCode::LoadImmNLong => {
                    let imm = self.get_std_or_long_operand(OpCode::LoadImmN) as i64;
                    self.push_stack(Object::Int(imm))
                }

                OpCode::LoadConstant | OpCode::LoadConstantLong => {
                    let pos = self.get_std_or_long_operand(OpCode::LoadConstant);
                    let val = self.read_constant(pos).clone();
                    self.push_stack(val)
                }

                OpCode::DefineGlobal | OpCode::DefineGlobalLong => {
                    let pos = self.get_std_or_long_operand(OpCode::DefineGlobal);

                    // Gets the name from the pool assigns the value to the global
                    if let Object::String(name) = self.read_constant(pos).clone() {
                        let val = self.pop_stack();
                        self.globals.insert(name, val);
                    } else {
                        unreachable!("Expected a string for global declaration name.");
                    }
                }

                OpCode::GetGlobal | OpCode::GetGlobalLong => {
                    let pos = self.get_std_or_long_operand(OpCode::GetGlobal);

                    // Gets the name from the pool
                    if let Object::String(name) = self.read_constant(pos).clone() {
                        let val = self.globals.get(&name).unwrap().clone();
                        self.push_stack(val);
                    } else {
                        unreachable!("Expected a string as global declaration name.");
                    }
                }

                OpCode::SetGlobal | OpCode::SetGlobalLong => {
                    let pos = self.get_std_or_long_operand(OpCode::SetGlobal);

                    // Gets the name from the pool
                    if let Object::String(name) = self.read_constant(pos).clone() {
                        let val = self.stack.last().unwrap().clone();
                        self.globals.insert(name, val);
                    } else {
                        unreachable!("Expected a string as global declaration name.");
                    }
                }

                OpCode::MakeIter => {
                    let tos = self.pop_stack();

                    match make_iter(tos) {
                        Ok(iter) => self.push_stack(iter),
                        Err(e) => return e,
                    }
                }

                OpCode::ForLoopIterNext => {
                    let tos = self.peek_stack(self.stack.len() - 1);

                    match tos {
                        Object::Iter(iter) => match get_next_in_iter(iter) {
                            Ok(o) => self.push_stack(o),
                            Err(_) => unreachable!("No more items to iterate in for-loop"),
                        },
                        _ => unreachable!("Cannot iterate object of type '{}'.", tos.type_name()),
                    }
                }

                OpCode::JumpHasNextOrPop | OpCode::JumpHasNextOrPopLong => {
                    let jump = self.get_std_or_long_operand(OpCode::JumpHasNextOrPop);

                    match self.peek_stack(self.stack.len() - 1) {
                        Object::Iter(o) => {
                            if iter_has_next(o) {
                                self.current_frame_mut().ip -= jump;
                            } else {
                                self.pop_stack();
                            }
                        }
                        _ => unreachable!("Expected iterable object on TOS."),
                    };
                }

                OpCode::MakeArray | OpCode::MakeArrayLong => {
                    // The number of values to pop from the stack. Essentially the size of the array.
                    let size = self.get_std_or_long_operand(OpCode::MakeArray);
                    let mut arr_values: Vec<Object> = vec![];

                    for _ in 0..size {
                        arr_values.push(self.pop_stack());
                    }

                    let arr = Rc::new(RefCell::new(arr_values));
                    self.push_stack(Object::Array(arr));
                }

                OpCode::MakeTuple | OpCode::MakeTupleLong => {
                    // The number of values to pop from the stack. Essentially the size of the array.
                    let size = self.get_std_or_long_operand(OpCode::MakeTuple);
                    let mut tuple_values: Vec<Object> = Vec::with_capacity(size);

                    for _ in 0..size {
                        tuple_values.push(self.pop_stack());
                    }

                    let tup = Box::new(TupleObject { tup: tuple_values });
                    self.push_stack(Object::Tuple(tup));
                }

                OpCode::Indexing => {
                    let index = self.pop_stack();
                    let target = self.pop_stack();

                    match target.get_at_index(&index) {
                        Ok(r) => self.push_stack(r),
                        Err(e) => return e.to_runtime_error(),
                    }
                }

                OpCode::GetLocal | OpCode::GetLocalLong => {
                    // The position of the local variable's value in the stack
                    let pos = self.get_std_or_long_operand(OpCode::GetLocal);

                    let idx = self.current_frame().base_pointer + pos;
                    let value = self.peek_stack(idx).clone();
                    self.push_stack(value);
                }

                OpCode::SetLocal | OpCode::SetLocalLong => {
                    // The position of the local variable's value in the stack
                    let pos = self.get_std_or_long_operand(OpCode::SetLocal);

                    let value = self.stack.last().unwrap();
                    let offset = self.current_frame().base_pointer;

                    self.stack[pos + offset] = value.clone();
                }

                OpCode::Negate => match -self.pop_stack() {
                    Ok(r) => self.push_stack(r),
                    Err(e) => return e.to_runtime_error(),
                },

                OpCode::Add => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();

                    match val1 + val2 {
                        Ok(r) => self.push_stack(r),
                        Err(e) => return e.to_runtime_error(),
                    }
                }

                OpCode::Subtract => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();

                    match val1 - val2 {
                        Ok(r) => self.push_stack(r),
                        Err(e) => return e.to_runtime_error(),
                    }
                }

                OpCode::Multiply => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();

                    match val1 * val2 {
                        Ok(r) => self.push_stack(r),
                        Err(e) => return e.to_runtime_error(),
                    }
                }

                OpCode::Divide => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();

                    match val1 / val2 {
                        Ok(r) => self.push_stack(r),
                        Err(e) => return e.to_runtime_error(),
                    }
                }

                OpCode::Modulus => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();

                    match val1 % val2 {
                        Ok(r) => self.push_stack(r),
                        Err(e) => return e.to_runtime_error(),
                    }
                }

                OpCode::Expo => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();

                    match val1.pow(val2) {
                        Ok(r) => self.push_stack(r),
                        Err(e) => return e.to_runtime_error(),
                    }
                }

                OpCode::LogicNot => {
                    let val = self.pop_stack();
                    self.push_stack(Object::Bool(val.is_falsey()));
                }

                OpCode::Equals => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();
                    self.push_stack(Object::Bool(val1.equals(&val2)));
                }

                OpCode::NotEq => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();
                    self.push_stack(Object::Bool(!val1.equals(&val2)));
                }

                OpCode::LessThan => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();

                    match val1.lt(val2) {
                        Ok(r) => self.push_stack(r),
                        Err(e) => return e.to_runtime_error(),
                    }
                }

                OpCode::LessThanEq => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();

                    match val1.lteq(val2) {
                        Ok(r) => self.push_stack(r),
                        Err(e) => return e.to_runtime_error(),
                    }
                }

                OpCode::GreaterThan => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();

                    match val1.gt(val2) {
                        Ok(r) => self.push_stack(r),
                        Err(e) => return e.to_runtime_error(),
                    }
                }

                OpCode::GreaterThanEq => {
                    let val2 = self.pop_stack();
                    let val1 = self.pop_stack();

                    match val1.gteq(val2) {
                        Ok(r) => self.push_stack(r),
                        Err(e) => return e.to_runtime_error(),
                    }
                }

                OpCode::BitwiseOr => {
                    let right = self.pop_stack();
                    let left = self.pop_stack();

                    match left | right {
                        Ok(r) => self.push_stack(r),
                        Err(e) => return e.to_runtime_error(),
                    }
                }

                OpCode::BitwiseXor => {
                    let right = self.pop_stack();
                    let left = self.pop_stack();

                    match left ^ right {
                        Ok(r) => self.push_stack(r),
                        Err(e) => return e.to_runtime_error(),
                    }
                }

                OpCode::BitwiseAnd => {
                    let right = self.pop_stack();
                    let left = self.pop_stack();

                    match left & right {
                        Ok(r) => self.push_stack(r),
                        Err(e) => return e.to_runtime_error(),
                    }
                }

                OpCode::BitwiseNot => match !self.pop_stack() {
                    Ok(r) => self.push_stack(r),
                    Err(e) => return e.to_runtime_error(),
                },

                OpCode::BitwiseShiftLeft => {
                    let right = self.pop_stack();
                    let left = self.pop_stack();

                    match left << right {
                        Ok(r) => self.push_stack(r),
                        Err(e) => return e.to_runtime_error(),
                    }
                }

                OpCode::BitwiseShiftRight => {
                    let right = self.pop_stack();
                    let left = self.pop_stack();

                    match left >> right {
                        Ok(r) => self.push_stack(r),
                        Err(e) => return e.to_runtime_error(),
                    }
                }

                OpCode::MakeRange => {
                    let right = self.pop_stack();
                    let left = self.pop_stack();

                    if left.is_int() && right.is_int() {
                        let a = left.as_int().unwrap();
                        let b = right.as_int().unwrap();
                        self.push_stack(Object::Range(RangeObject { min: a, max: b }));
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

                OpCode::NullishCoalescing => {
                    let value = self.pop_stack();
                    let nullish = self.pop_stack();

                    if nullish.is_null() {
                        self.push_stack(value);
                    } else {
                        self.push_stack(nullish);
                    }
                }

                OpCode::PopJumpIfFalse => {
                    // The POP_JUMP_IF_FALSE instruction always has a short as its operand.
                    let offset = self.get_next_short() as usize;

                    if self.pop_stack().is_falsey() {
                        self.current_frame_mut().ip += offset;
                    }
                }

                OpCode::JumpIfFalseOrPop => {
                    // The JUMP_IF_FALSE_OR_POP instruction always has a short as its operand.
                    let offset = self.get_next_short() as usize;

                    if self.peek_stack(self.stack.len() - 1).is_falsey() {
                        self.current_frame_mut().ip += offset;
                    } else {
                        self.pop_stack();
                    }
                }

                OpCode::JumpIfTrueOrPop => {
                    // The JUMP_IF_TRUE_OR_POP instruction always has a short as its operand.
                    let offset = self.get_next_short() as usize;

                    if !self.peek_stack(self.stack.len() - 1).is_falsey() {
                        self.current_frame_mut().ip += offset;
                    } else {
                        self.pop_stack();
                    }
                }

                OpCode::JumpForward => {
                    // The JUMP_FORWARD instruction always has a short as its operand.
                    let offset = self.get_next_short() as usize;
                    self.current_frame_mut().ip += offset;
                }

                OpCode::LoopJump | OpCode::LoopJumpLong => {
                    let offset = self.get_std_or_long_operand(OpCode::LoopJump);
                    self.current_frame_mut().ip -= offset;
                }

                OpCode::LoadNative => {
                    let native = self.get_next_byte() as usize;

                    match self.natives.get_native_fn_object(native) {
                        Ok(f) => self.push_stack(Object::Native(Box::new(f))),
                        Err(e) => return e,
                    }
                }

                OpCode::FuncCall => {
                    // Functions can only have 255-MAX parameters
                    let arg_count = self.get_next_byte();

                    let maybe_function = self
                        .peek_stack(self.stack.len() - (arg_count as usize) - 1)
                        .clone();

                    match self.call_value(maybe_function, arg_count) {
                        RuntimeResult::Ok => {}
                        RuntimeResult::Error { error, message } => {
                            return RuntimeResult::Error { error, message }
                        }
                    }
                }

                OpCode::MakeClosure | OpCode::MakeClosureLarge => {
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
                            self.create_up_value(self.current_frame().base_pointer + index)
                        } else {
                            self.current_frame().closure.up_values[index].clone()
                        };

                        up_values.push(up.clone());
                    }

                    self.push_stack(Object::Closure(ClosureObject { function, up_values }));
                }

                OpCode::MakeClosureLong | OpCode::MakeClosureLongLarge => {
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
                            self.create_up_value(self.current_frame().base_pointer + index)
                        } else {
                            self.current_frame().closure.up_values[index].clone()
                        };

                        up_values.push(up.clone());
                    }

                    self.push_stack(Object::Closure(ClosureObject { function, up_values }));
                }

                OpCode::GetUpVal | OpCode::GetUpValLong => {
                    let pos = self.get_std_or_long_operand(OpCode::GetUpVal);

                    let val = match &*self.get_up_val(pos).borrow() {
                        UpValRef::Open(l) => self.peek_stack(*l).clone(),
                        UpValRef::Closed(o) => o.clone(),
                    };

                    self.push_stack(val);
                }

                OpCode::SetUpVal | OpCode::SetUpValLong => {
                    let pos = self.get_std_or_long_operand(OpCode::SetUpVal);
                    let new_val = self.stack.last().unwrap().clone();

                    match &mut *self.get_up_val(pos).borrow_mut() {
                        UpValRef::Open(l) => self.stack[*l] = new_val,
                        UpValRef::Closed(u) => *u = new_val,
                    }
                }

                OpCode::CloseUpVal | OpCode::CloseUpValLong => {
                    let pos = self.get_std_or_long_operand(OpCode::CloseUpVal);

                    for u in self.up_values.iter() {
                        if u.borrow().is_open_at(self.current_frame().base_pointer + pos) {
                            let new_val = self.peek_stack(self.current_frame().base_pointer + pos);
                            u.replace(UpValRef::Closed(new_val.clone()));
                            break;
                        }
                    }
                }

                OpCode::PopCloseUpVal => {
                    let new_val = self.pop_stack();

                    for u in self.up_values.iter() {
                        if u.borrow().is_open_at(self.stack.len()) {
                            u.replace(UpValRef::Closed(new_val));
                            break;
                        }
                    }
                }

                OpCode::BindDefaults => {
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
                }

                OpCode::Return => {
                    let result = self.pop_stack();
                    let locals_to_pop = self.stack.len() - self.current_frame().base_pointer;

                    // Pops local declarations from the stack
                    for _ in 0..locals_to_pop {
                        self.pop_stack();
                    }

                    // removes the call frame
                    self.frames.pop();
                    self.push_stack(result);
                }

                OpCode::MakeClass | OpCode::MakeClassLong => {
                    let pos = self.get_std_or_long_operand(OpCode::MakeClass);

                    let name = match self.read_constant(pos).clone() {
                        Object::String(s) => s,
                        _ => unreachable!("Expected string for class name."),
                    };

                    let new_class = Object::Class(ClassObject { name });
                    self.push_stack(new_class);
                }

                OpCode::MakeInstance => {
                    // Instances can only have 255-MAX arguments
                    let arg_count = self.get_next_byte();

                    let maybe_instance = self
                        .peek_stack(self.stack.len() - (arg_count as usize) - 1)
                        .clone();

                    match self.create_instance(maybe_instance, arg_count) {
                        RuntimeResult::Ok => {}
                        RuntimeResult::Error { error, message } => {
                            return RuntimeResult::Error { error, message }
                        }
                    }
                }

                OpCode::GetProp | OpCode::GetPropLong => {
                    let pos = self.get_std_or_long_operand(OpCode::GetProp);

                    let prop_name = match self.read_constant(pos).clone() {
                        Object::String(name) => name,
                        _ => unreachable!("Expected string for 'GetProp' name."),
                    };

                    match self.pop_stack() {
                        Object::Instance(x) => {
                            if x.borrow().fields.contains_key(&prop_name) {
                                let val = x.borrow().fields.get(&prop_name).unwrap().clone();
                                self.push_stack(val);
                            } else {
                                return RuntimeResult::Error {
                                    error: RuntimeErrorType::ReferenceError,
                                    message: format!(
                                        "Property '{}' not defined for object of type '{}'.",
                                        prop_name,
                                        x.borrow().class.name
                                    ),
                                };
                            }
                        }
                        _ => todo!("Other objects also have properties."),
                    }
                }

                OpCode::SetProp | OpCode::SetPropLong => {
                    let pos = self.get_std_or_long_operand(OpCode::SetProp);

                    let prop_name = match self.read_constant(pos).clone() {
                        Object::String(name) => name,
                        _ => unreachable!("Expected string for 'SetProp' name."),
                    };

                    let value = self.pop_stack();

                    match self.pop_stack() {
                        // TODO: This does not actually modify the instance object assigned to
                        // a variable. It only modifies the cloned instance that is currently
                        // on top of the stack. We need a heap to store class instances.
                        Object::Instance(x) => {
                            x.borrow_mut().fields.insert(prop_name, value.clone());
                            self.push_stack(value);
                        }
                        _ => todo!("Other objects also have properties."),
                    }
                }

                OpCode::EndVirtualMachine => {
                    self.pop_stack(); // Remove the main function off the stack
                    self.frames.pop();
                    return RuntimeResult::Ok;
                }
            }

            // Prints the execution of the program.
            // self.print_execution(instruction);
        }
    }
}
