use crate::bytecode::OpCode;
use crate::compiler::Compiler;
use crate::errors::{report_errors_list, report_runtime_error, RuntimeErrorType};
use crate::natives::Natives;
use crate::objects::{ClosureObject, FuncObject, InstanceObject, Object, UpValRef};
use crate::parser::Parser;
use crate::FRAMES_MAX;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

// Submodules
mod run;

/// Represents a single ongoing function call.
pub struct CallFrame {
   /// The closure (function) for this call frame.
   pub closure: ClosureObject,
   /// The index of the current instruction being executed.
   pub ip: usize,
   /// The stack index for the base of this call frame.
   pub return_index: usize,
}

impl CallFrame {
   /// Gets the current instruction without incrementing the instruction pointer.
   fn peek_current_op_code(&self) -> OpCode {
      self.closure.function.borrow().chunk.get_op_code(self.ip - 1)
   }

   /// Gets the current instruction and advances the instruction pointer to the next instruction.
   fn get_next_op_code(&mut self) -> OpCode {
      let code = self.closure.function.borrow().chunk.get_op_code(self.ip);
      self.ip += 1;
      return code;
   }

   /// Gets the current raw byte and advances the instruction pointer to the next instruction.
   fn get_next_byte(&mut self) -> u8 {
      let code = self.closure.function.borrow().chunk.get_byte(self.ip);
      self.ip += 1;
      return code;
   }

   /// Gets the current two raw bytes and advances the instruction pointer by 2 instructions.
   fn get_next_short(&mut self) -> u16 {
      let next_short = self.closure.function.borrow().chunk.get_short(self.ip);
      self.ip += 2;
      return next_short;
   }

   /// Gets an object from the current call frame's constant pool.
   fn get_constant(&self, idx: usize) -> Object {
      self.closure.function.borrow().chunk.get_constant(idx).clone()
   }
}

/// Represents a virtual machine.
pub struct VirtualMachine {
   /// The path to the source file.
   filepath: PathBuf,
   /// A list of call frames (the VM's call frames stack).
   frames: Vec<CallFrame>,
   /// A list of temporary objects (the VM's values stack).
   stack: Vec<Object>,
   /// The global declarations made in the program.
   globals: HashMap<String, Object>,
   /// A collection of UpValues in the program.
   /// TODO: Find a better way to manage UpValues.
   up_values: Vec<Rc<RefCell<UpValRef>>>,
   /// The native functions available in a Hinton program.
   natives: Natives,
}

/// The types of results the interpreter can return.
pub enum InterpretResult {
   CompileError,
   Ok,
   ParseError,
   RuntimeError,
}

/// Represents the internal state of the interpreter after some computation.
pub enum RuntimeResult {
   Error {
      error: RuntimeErrorType,
      message: String,
   },
   EndOK,
   Continue,
}

impl VirtualMachine {
   /// Interprets the source text of a file.
   ///
   /// # Returns
   /// - `InterpretResult`: The result of the source interpretation.
   pub fn interpret(filepath: PathBuf, source: &str) -> InterpretResult {
      // Creates a new virtual machine
      let mut _self = VirtualMachine {
         stack: Vec::with_capacity(256),
         frames: Vec::with_capacity(256),
         filepath,
         globals: Default::default(),
         up_values: vec![],
         natives: Natives::default(),
      };

      // Parses the program into an AST and aborts if there are any parsing errors.
      let ast = match Parser::parse(source) {
         Ok(x) => Rc::new(x),
         Err(e) => {
            report_errors_list(&_self.filepath, e, source);
            return InterpretResult::ParseError;
         }
      };

      // Compiles the program into bytecode and aborts if there are any compiling errors.
      let module = match Compiler::compile_ast(&_self.filepath, &ast, _self.natives.get_names()) {
         Ok(x) => x,
         Err(e) => {
            report_errors_list(&_self.filepath, e, source);
            return InterpretResult::CompileError;
         }
      };

      let f = Rc::new(RefCell::new(module));
      _self.stack.push(Object::Function(f.clone()));

      return match _self.call_function(f, 0) {
         RuntimeResult::Continue => {
            // Runs the program.
            match _self.run() {
               RuntimeResult::EndOK => InterpretResult::Ok,
               RuntimeResult::Error { error, message } => {
                  report_runtime_error(&_self, error, message, source);
                  InterpretResult::RuntimeError
               }
               RuntimeResult::Continue => unreachable!(),
            }
         }
         RuntimeResult::Error { error, message } => {
            report_runtime_error(&_self, error, message, source);
            InterpretResult::RuntimeError
         }
         RuntimeResult::EndOK => unreachable!(),
      };
   }

   /// Gets a reference to the call frames stack.
   pub fn frames_stack(&self) -> &Vec<CallFrame> {
      &self.frames
   }

   /// Gets an immutable reference to the current call frame.
   pub fn current_frame(&self) -> &CallFrame {
      self.frames.last().unwrap()
   }

   /// Gets a mutable reference to the current call frame.
   fn current_frame_mut(&mut self) -> &mut CallFrame {
      let frames_len = self.frames.len();
      &mut self.frames[frames_len - 1]
   }

   /// Gets the next OpCode in the chunk.
   fn get_next_op_code(&mut self) -> OpCode {
      self.current_frame_mut().get_next_op_code()
   }

   /// Gets the next raw byte in the chunk.
   fn get_next_byte(&mut self) -> u8 {
      self.current_frame_mut().get_next_byte()
   }

   /// Gets the next raw two bytes in the chunk.
   fn get_next_short(&mut self) -> u16 {
      self.current_frame_mut().get_next_short()
   }

   /// Pops the last object in the objects stack.
   fn pop_stack(&mut self) -> Object {
      match self.stack.pop() {
         Some(obj) => obj,
         None => {
            panic!("Stack is empty!")
         }
      }
   }

   /// Pushes an object onto the back of the objects stack.
   fn push_stack(&mut self, new_val: Object) -> RuntimeResult {
      self.stack.push(new_val);
      RuntimeResult::Continue
   }

   /// Gets an immutable reference to the object at the provided stack index.
   fn peek_stack(&self, pos: usize) -> &Object {
      &self.stack[pos]
   }

   /// Gets a mutable reference to the object at the provided stack index.
   fn peek_stack_mut(&mut self, pos: usize) -> &mut Object {
      &mut self.stack[pos]
   }

   /// Gets an UpValue from the UpValues list.
   pub fn get_up_val(&self, idx: usize) -> Rc<RefCell<UpValRef>> {
      self.current_frame().closure.up_values[idx].clone()
   }

   /// Gets an object from the current call frame's constant pool.
   fn read_constant(&self, idx: usize) -> Object {
      return self.current_frame().get_constant(idx);
   }

   /// Either gets the next byte, or the next short based on the instruction.
   /// If the current instruction matches the instruction corresponding to a one-byte operand,
   /// then this function returns the next byte as `usize`, otherwise it will return the next
   /// two bytes a `usize`.
   fn get_std_or_long_operand(&mut self, op: OpCode) -> usize {
      // The compiler makes sure that the structure of the bytecode is correct
      // for the VM to execute, so unwrapping without check should be fine.
      if op == self.current_frame_mut().peek_current_op_code() {
         self.get_next_byte() as usize
      } else {
         self.get_next_short() as usize
      }
   }

   /// Tries to call the given object, or returns a runtime error if the object is not callable.
   fn call_object(&mut self, callee: Object, arg_count: u8) -> RuntimeResult {
      return match callee {
         Object::Function(obj) => self.call_function(obj, arg_count),
         Object::Closure(obj) => self.call_closure(obj, arg_count),
         Object::Native(obj) => {
            let mut args: Vec<Object> = vec![];
            for _ in 0..arg_count {
               let val = self.pop_stack();
               args.push(val);
            }
            args.reverse();

            match self.natives.call_native(&obj.name, args) {
               Ok(x) => {
                  // Pop the native function call off the stack
                  self.pop_stack();
                  // Place the result of the call on top of the stack
                  self.push_stack(x);
                  RuntimeResult::Continue
               }
               Err(e) => e,
            }
         }
         _ => RuntimeResult::Error {
            error: RuntimeErrorType::TypeError,
            message: format!("Cannot call object of type '{}'.", callee.type_name()),
         },
      };
   }

   /// Tries to call a function object, or returns a runtime error is there was a problem while
   /// creating the function's call frame.
   fn call_function(&mut self, callee: Rc<RefCell<FuncObject>>, arg_count: u8) -> RuntimeResult {
      if let Err(e) = self.verify_call(&callee.borrow(), arg_count) {
         return e;
      }

      let max_arity = callee.borrow().max_arity as usize;

      self.frames.push(CallFrame {
         closure: ClosureObject {
            function: callee,
            up_values: vec![],
         },
         ip: 0,
         return_index: self.stack.len() - max_arity - 1,
      });

      RuntimeResult::Continue
   }

   /// Tries to call a closure object, or returns a runtime error is there was a problem while
   /// creating the closure's call frame.
   fn call_closure(&mut self, callee: ClosureObject, arg_count: u8) -> RuntimeResult {
      if let Err(e) = self.verify_call(&callee.function.borrow(), arg_count) {
         return e;
      }

      let max_arity = callee.function.borrow().max_arity as usize;

      self.frames.push(CallFrame {
         closure: callee,
         ip: 0,
         return_index: self.stack.len() - max_arity - 1,
      });

      RuntimeResult::Continue
   }

   /// Verifies that a function or closure can be called with the provided number of arguments,
   /// and that the maximum recursion depth has not been exceeded.
   fn verify_call(&mut self, function: &FuncObject, arg_count: u8) -> Result<(), RuntimeResult> {
      let max_arity = function.max_arity;
      let min_arity = function.min_arity;

      // Check that the correct number of arguments is passed to the function
      if arg_count < min_arity || arg_count > max_arity {
         let msg;

         if min_arity == max_arity {
            msg = format!("Expected {} arguments but got {} instead.", min_arity, arg_count);
         } else {
            msg = format!(
               "Expected {} to {} arguments but got {} instead.",
               min_arity, max_arity, arg_count
            );
         };

         return Err(RuntimeResult::Error {
            error: RuntimeErrorType::ArgumentError,
            message: msg,
         });
      }

      // Pushes the default values onto the stack
      // if they were not passed into the func call
      if arg_count < max_arity {
         let missing_args = (max_arity - arg_count) as usize;
         let def_count = function.defaults.len();

         for i in (def_count - missing_args)..def_count {
            let val = function.defaults[i as usize].clone();
            self.push_stack(val);
         }
      }

      // Check we are not overflowing the stack of frames
      if self.frames.len() >= (FRAMES_MAX as usize) {
         return Err(RuntimeResult::Error {
            error: RuntimeErrorType::RecursionError,
            message: String::from("Maximum recursion depth exceeded."),
         });
      }

      Ok(())
   }

   /// Creates an OpenUpValue from the given stack index.
   fn create_up_value(&mut self, index: usize) -> Rc<RefCell<UpValRef>> {
      for u in self.up_values.iter() {
         if u.borrow().is_open_at(index) {
            return u.clone();
         }
      }

      let new_up_val = Rc::new(RefCell::new(UpValRef::Open(index)));
      self.up_values.push(new_up_val.clone());
      new_up_val
   }

   /// Tries to create a class instance with the given object, or returns a runtime error if the
   /// object is not instantiable.
   fn create_instance(&mut self, callee: Object, arg_count: u8) -> RuntimeResult {
      let inst = Rc::new(RefCell::new(InstanceObject {
         class: match callee {
            Object::Class(c) => c,
            _ => {
               return RuntimeResult::Error {
                  error: RuntimeErrorType::InstanceError,
                  message: format!(
                     "Cannot create an instance from an object of type '{}'.",
                     callee.type_name()
                  ),
               }
            }
         },
         fields: HashMap::new(),
      }));

      let new_instance = Object::Instance(inst);

      let class_pos = self.stack.len() - (arg_count as usize) - 1;
      self.stack[class_pos] = new_instance;

      RuntimeResult::Continue
   }

   /// Prints the execution trace for the program. Useful for debugging the VM.
   #[allow(dead_code)]
   fn print_execution(&mut self, instr: OpCode) {
      println!("\n==========================");

      // Prints the next instruction to be executed
      println!("OpCode:\t\x1b[36m{:?}\x1b[0m ", instr);
      println!("Byte:\t{:#04X} ", instr as u8);

      // Prints the index of the current instruction
      println!("IP:\t{:>04} ", self.current_frame().ip);

      // Prints the current state of the values stack
      print!("stack\t[");
      for val in self.stack[1..].iter() {
         print!("{}; ", val);
      }
      println!("]");

      print!("Output:\t");
   }
}
