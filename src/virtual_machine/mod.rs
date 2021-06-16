#[cfg(feature = "bench_time")]
use std::time::Instant;

use crate::{
    bytecode::OpCode,
    compiler::Compiler,
    errors::{report_errors_list, report_runtime_error, RuntimeErrorType},
    exec_time,
    natives::Natives,
    objects::{self, ClosureObject, FuncObject, InstanceObject, Object, UpValRef},
    parser::Parser,
    FRAMES_MAX,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

// Submodules
mod run;

/// The types of results the interpreter can return
pub enum InterpretResult {
    CompileError,
    Ok,
    ParseError,
    RuntimeError,
}

/// Represents a single ongoing function call.
pub struct CallFrame {
    pub closure: ClosureObject,
    pub ip: usize,
    pub base_pointer: usize,
}

impl CallFrame {
    fn peek_current_op_code(&self) -> OpCode {
        self.closure.function.borrow().chunk.get_op_code(self.ip - 1)
    }

    fn get_next_op_code(&mut self) -> OpCode {
        let code = self.closure.function.borrow().chunk.get_op_code(self.ip);
        self.ip += 1;
        return code;
    }

    fn get_next_byte(&mut self) -> u8 {
        let code = self.closure.function.borrow().chunk.get_byte(self.ip);
        self.ip += 1;
        return code;
    }

    fn get_next_short(&mut self) -> u16 {
        let next_short = self.closure.function.borrow().chunk.get_short(self.ip);
        self.ip += 2;
        return next_short;
    }

    fn get_constant(&self, idx: usize) -> Object {
        self.closure.function.borrow().chunk.get_constant(idx).clone()
    }
}

/// Represents a virtual machine
pub struct VirtualMachine {
    filepath: String,
    frames: Vec<CallFrame>,
    stack: Vec<objects::Object>,
    globals: HashMap<String, Object>,
    up_values: Vec<Rc<RefCell<UpValRef>>>,
    natives: Natives,
}

pub enum RuntimeResult {
    Error {
        error: RuntimeErrorType,
        message: String,
    },
    Ok,
}

impl VirtualMachine {
    /// Interprets a chuck of code.
    ///
    /// ## Returns
    /// * `InterpretResult` – The result of the source interpretation.
    pub fn interpret(filepath: &str, source: &str) -> InterpretResult {
        // Creates a new virtual machine
        let mut _self = VirtualMachine {
            stack: Vec::with_capacity(256),
            frames: Vec::with_capacity(256),
            filepath: String::from(filepath),
            globals: Default::default(),
            up_values: vec![],
            natives: Natives::default(),
        };

        // Parses the program into an AST and calculates the parser's execution time
        let parsing = exec_time(|| Parser::parse(source));

        // Aborts if there are any parsing errors
        let ast = match parsing.0 {
            Ok(x) => Rc::new(x),
            Err(e) => {
                report_errors_list(&_self.filepath, e, source);
                return InterpretResult::ParseError;
            }
        };

        // Compiles the program into bytecode and calculates the compiler's execution time
        let compiling = exec_time(|| Compiler::compile_file(filepath, &ast, _self.natives.names.clone()));
        let module = match compiling.0 {
            Ok(x) => x,
            Err(e) => {
                report_errors_list(&_self.filepath, e, source);
                return InterpretResult::CompileError;
            }
        };

        // Executes the program
        let f = Rc::new(RefCell::new(module));
        _self.stack.push(Object::Function(f.clone()));
        return match _self.call_function(f, 0) {
            RuntimeResult::Ok => {
                #[cfg(feature = "bench_time")]
                let start = Instant::now();

                // Runs the program.
                let runtime_result = _self.run();

                #[cfg(feature = "bench_time")]
                {
                    let run_time = start.elapsed();

                    println!("\n======= ⚠️  Execution Results ⚠️  =======");
                    println!("Parse Time:\t{:?}", parsing.1);
                    println!("Compile Time:\t{:?}", compiling.1);
                    println!("Run Time:\t{:?}", run_time);
                    println!("=======================================");
                }

                match runtime_result {
                    RuntimeResult::Ok => InterpretResult::Ok,
                    RuntimeResult::Error { error, message } => {
                        report_runtime_error(&_self, error, message, source);
                        InterpretResult::RuntimeError
                    }
                }
            }
            RuntimeResult::Error { error, message } => {
                report_runtime_error(&_self, error, message, source);
                InterpretResult::RuntimeError
            }
        };
    }

    pub fn frames_list(&self) -> &Vec<CallFrame> {
        &self.frames
    }

    pub fn current_frame(&self) -> &CallFrame {
        self.frames.last().unwrap()
    }

    pub fn get_up_val(&self, idx: usize) -> Rc<RefCell<UpValRef>> {
        self.current_frame().closure.up_values[idx].clone()
    }

    fn current_frame_mut(&mut self) -> &mut CallFrame {
        let frames_len = self.frames.len();
        &mut self.frames[frames_len - 1]
    }

    fn get_next_op_code(&mut self) -> OpCode {
        self.current_frame_mut().get_next_op_code()
    }

    fn get_next_byte(&mut self) -> u8 {
        self.current_frame_mut().get_next_byte()
    }

    fn get_next_short(&mut self) -> u16 {
        self.current_frame_mut().get_next_short()
    }

    fn pop_stack(&mut self) -> Object {
        match self.stack.pop() {
            Some(obj) => obj,
            None => {
                panic!("Stack is empty!")
            }
        }
    }

    fn push_stack(&mut self, new_val: Object) {
        self.stack.push(new_val)
    }

    fn peek_stack(&self, pos: usize) -> &Object {
        &self.stack[pos]
    }

    fn peek_stack_mut(&mut self, pos: usize) -> &mut Object {
        &mut self.stack[pos]
    }

    fn read_constant(&self, idx: usize) -> Object {
        return self.current_frame().get_constant(idx);
    }

    /// Either gets the next byte or the next short based on the instruction.
    /// If the current instruction matches the instruction corresponding to
    /// a one-byte operand, then this function returns the next byte as `usize`,
    /// otherwise it will return the next two bytes a `usize`.
    ///
    /// ## Arguments
    /// * `op` – The instructions corresponding to a one-byte operand.
    fn get_std_or_long_operand(&mut self, op: OpCode) -> usize {
        // The compiler makes sure that the structure of the bytecode is correct
        // for the VM to execute, so unwrapping without check should be fine.
        if op == self.current_frame_mut().peek_current_op_code() {
            self.get_next_byte() as usize
        } else {
            self.get_next_short() as usize
        }
    }

    fn call_value(&mut self, callee: Object, arg_count: u8) -> RuntimeResult {
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
                        RuntimeResult::Ok
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
            base_pointer: self.stack.len() - max_arity - 1,
        });

        RuntimeResult::Ok
    }

    fn call_closure(&mut self, callee: ClosureObject, arg_count: u8) -> RuntimeResult {
        if let Err(e) = self.verify_call(&callee.function.borrow(), arg_count) {
            return e;
        }

        let max_arity = callee.function.borrow().max_arity as usize;

        self.frames.push(CallFrame {
            closure: callee,
            ip: 0,
            base_pointer: self.stack.len() - max_arity - 1,
        });

        RuntimeResult::Ok
    }

    fn create_instance(&mut self, callee: Object, arg_count: u8) -> RuntimeResult {
        let inst = Rc::new(RefCell::new(InstanceObject {
            class: match callee {
                Object::Class(c) => c,
                _ => {
                    return RuntimeResult::Error {
                        error: RuntimeErrorType::InstanceError,
                        message: format!(
                            "Cannot create instance from object of type '{}'.",
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

        RuntimeResult::Ok
    }

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

    /// Prints the execution trace for the program. Useful for debugging the VM.
    ///
    /// ## Arguments
    /// * `instr` – The current OpCode to be executed.
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
