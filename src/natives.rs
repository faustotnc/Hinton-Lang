use crate::errors::RuntimeErrorType;
use crate::objects::{IterObject, NativeFuncObj, Object};
use crate::virtual_machine::RuntimeResult;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io;
use std::rc::Rc;
use std::time::SystemTime;

/// Represents the body of a Hinton native function object.
pub type NativeFn = fn(Vec<Object>) -> Result<Object, RuntimeResult>;

/// Represents the list of native functions available through a Hinton program.
pub struct Natives(HashMap<String, NativeFuncObj>);

/// The default implementation of a native function list.
impl Default for Natives {
   fn default() -> Self {
      let mut natives = Natives(Default::default());

      // >>>>>>>>>>>>>>>> Native functions to be added after this line
      natives.add_native_function("assert", 1, 2, native_assert as NativeFn);
      natives.add_native_function("assert_eq", 2, 3, native_assert_eq as NativeFn);
      natives.add_native_function("assert_ne", 2, 3, native_assert_ne as NativeFn);
      natives.add_native_function("clock", 0, 0, native_clock as NativeFn);
      natives.add_native_function("input", 1, 1, native_input as NativeFn);
      natives.add_native_function("iter", 1, 1, native_iter as NativeFn);
      natives.add_native_function("next", 1, 1, native_next as NativeFn);
      natives.add_native_function("print", 1, 1, native_print as NativeFn);
      // <<<<<<<<<<<<<<<< Native functions to be added before this line

      return natives;
   }
}

impl Natives {
   /// Adds a native function definition to the native functions list.
   fn add_native_function(&mut self, name: &str, min_arity: u8, max_arity: u8, body: NativeFn) {
      let name = String::from(name);

      if !self.0.contains_key(&name) {
         let f = NativeFuncObj {
            name: name.clone(),
            min_arity,
            max_arity,
            function: body,
         };

         self.0.insert(name.clone(), f);
      } else {
         panic!("Cannot duplicate native function '{}'.", name);
      }
   }

   /// Finds and executes a native function by name.
   pub fn call_native(&mut self, name: &str, args: Vec<Object>) -> Result<Object, RuntimeResult> {
      match self.0.get(name) {
         Some(f) => {
            let args_len = args.len() as u8;

            // Checks the argument arity for the function call.
            if args_len < f.min_arity || args_len > f.max_arity {
               return if f.min_arity == f.max_arity {
                  Err(RuntimeResult::Error {
                     error: RuntimeErrorType::ArgumentError,
                     message: format!("Expected {} arguments but got {} instead.", f.min_arity, args_len),
                  })
               } else {
                  Err(RuntimeResult::Error {
                     error: RuntimeErrorType::ArgumentError,
                     message: format!(
                        "Expected {} to {} arguments but got {} instead.",
                        f.min_arity, f.max_arity, args_len
                     ),
                  })
               };
            }

            // Calls the native function
            let call_result = (f.function)(args);

            // Returns the result of the call
            return call_result;
         }
         None => Err(RuntimeResult::Error {
            error: RuntimeErrorType::ReferenceError,
            message: format!("No native function named '{}'.", name),
         }),
      }
   }

   /// Obtains the NativeFunctionObj associated with a native function name.
   pub fn get_native_fn_object(&self, idx: usize) -> Result<NativeFuncObj, RuntimeResult> {
      let name = self.0.keys().collect::<Vec<&String>>()[idx];

      match self.0.get(name) {
         Some(f) => Ok(f.to_owned()),
         None => Err(RuntimeResult::Error {
            error: RuntimeErrorType::ReferenceError,
            message: format!("No native function named '{}'.", name),
         }),
      }
   }

   /// Obtains a list of the names of the native functions
   pub fn get_names(&self) -> Vec<String> {
      self.0.keys().map(|x| x.clone()).collect()
   }
}

/// >>>>>>>>>>>>>>>>> =============================================== <<<<<<<<<<<<<<<<<<<
/// ================= Native Function Implementations After This Line ===================
/// >>>>>>>>>>>>>>>>> =============================================== <<<<<<<<<<<<<<<<<<<

/// Implements the `print(...)` native function for Hinton,
/// which prints a value to the console.
fn native_print(args: Vec<Object>) -> Result<Object, RuntimeResult> {
   println!("{}", args[0]);
   Ok(Object::Null)
}

/// Implements the `clock()` native function for Hinton, which
/// retrieves the current time from the Unix Epoch time.
fn native_clock(_: Vec<Object>) -> Result<Object, RuntimeResult> {
   let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);

   match now {
      Ok(t) => {
         let time = t.as_millis();
         Ok(Object::Int(time as i64))
      }
      Err(_) => Err(RuntimeResult::Error {
         error: RuntimeErrorType::Internal,
         message: String::from("System's time before UNIX EPOCH."),
      }),
   }
}

/// Implements the `iter(...)` native function for Hinton, which
/// converts the give object to an iterable.
fn native_iter(args: Vec<Object>) -> Result<Object, RuntimeResult> {
   make_iter(args[0].clone())
}

/// Converts a Hinton object into an Iterable.
pub fn make_iter(o: Object) -> Result<Object, RuntimeResult> {
   match o {
      Object::String(_) => {}
      Object::Array(_) => {}
      Object::Range(_) => {}
      Object::Tuple(_) => {}
      // If the object is already an iterable, return that same object.
      Object::Iter(_) => return Ok(o),
      // Object cannot be iterable.
      _ => {
         return Err(RuntimeResult::Error {
            error: RuntimeErrorType::TypeError,
            message: format!("Cannot create iterable from '{}'.", o.type_name()),
         })
      }
   };

   return Ok(Object::Iter(Rc::new(RefCell::new(IterObject {
      iter: Box::new(o),
      index: 0,
   }))));
}

/// Implements the `next(...)` native function for Hinton, which
/// retrieves the next item in an iterable object.
fn native_next(args: Vec<Object>) -> Result<Object, RuntimeResult> {
   match &args[0] {
      Object::Iter(iter) => get_next_in_iter(iter),
      _ => Err(RuntimeResult::Error {
         error: RuntimeErrorType::TypeError,
         message: format!("Object of type '{}' is not iterable.", args[0].type_name()),
      }),
   }
}

/// Gets the next item in a Hinton iterator.
pub fn get_next_in_iter(o: &Rc<RefCell<IterObject>>) -> Result<Object, RuntimeResult> {
   let mut iter = o.borrow_mut();
   let current_index = Object::Int(iter.index as i64);

   // Since we are passing an integer into the `Object.get(...)` method,
   // the only error that can occur is an `IndexOutOfBounds` error, which
   // in terms of iterators means there are no more items left to iterate.
   let obj = match iter.iter.subscript(&current_index) {
      Ok(o) => o,
      Err(_) => {
         return Err(RuntimeResult::Error {
            error: RuntimeErrorType::StopIteration,
            message: String::from("End of Iterator."),
         })
      }
   };

   // Increment to the next position of the iterator.
   iter.index += 1;

   return Ok(obj);
}

/// Implements the `input(...)` native function for Hinton, which
/// gets user input from the console.
fn native_input(args: Vec<Object>) -> Result<Object, RuntimeResult> {
   print!("{}", args[0]);

   // Print the programmer-provided message
   match io::Write::flush(&mut io::stdout()) {
      Ok(_) => {
         let mut input = String::new();
         // Get the user's input
         match io::stdin().read_line(&mut input) {
            Ok(_) => {
               input.pop(); // remove added newline
               Ok(Object::String(input))
            }
            Err(e) => Err(RuntimeResult::Error {
               error: RuntimeErrorType::Internal,
               message: format!("Failed to read input. IO failed read line. {}", e),
            }),
         }
      }
      Err(e) => Err(RuntimeResult::Error {
         error: RuntimeErrorType::Internal,
         message: format!("Failed to read input. IO failed flush. {}", e),
      }),
   }
}

// Implements the `assert(...)` native function for Hinton, which checks that
// the first argument of the function call is truthy, emitting a RuntimeError
// (with an optional third parameter as its message) if the value is falsey.
fn native_assert(args: Vec<Object>) -> Result<Object, RuntimeResult> {
   let value = args[0].clone();

   if !value.is_falsey() {
      Ok(Object::Null)
   } else {
      let message = if args.len() == 2 {
         args[1].clone()
      } else {
         Object::String(String::from("Assertion failed on a falsey value."))
      };

      Err(RuntimeResult::Error {
         error: RuntimeErrorType::AssertionError,
         message: format!("{}", message),
      })
   }
}

// Implements the `assert_eq(...)` native function for Hinton, which checks that
// the first two arguments of the function call are equal, emitting a RuntimeError
// (with an optional third parameter as its message) if the values are not equal.
fn native_assert_eq(args: Vec<Object>) -> Result<Object, RuntimeResult> {
   let value1 = args[0].clone();
   let value2 = args[1].clone();

   if value1.equals(&value2) {
      Ok(Object::Null)
   } else {
      let message = if args.len() == 3 {
         args[2].clone()
      } else {
         Object::String(String::from("Assertion values are not equal."))
      };

      Err(RuntimeResult::Error {
         error: RuntimeErrorType::AssertionError,
         message: format!("{}", message),
      })
   }
}

// Implements the `assert_ne(...)` native function for Hinton, which checks that
// the first two arguments of the function call are not equal, emitting a RuntimeError
// (with an optional third parameter as its message) if the values are equal.
fn native_assert_ne(args: Vec<Object>) -> Result<Object, RuntimeResult> {
   let value1 = args[0].clone();
   let value2 = args[1].clone();

   if !value1.equals(&value2) {
      Ok(Object::Null)
   } else {
      let message = if args.len() == 3 {
         args[2].clone()
      } else {
         Object::String(String::from("Assertion values are equal."))
      };

      Err(RuntimeResult::Error {
         error: RuntimeErrorType::AssertionError,
         message: format!("{}", message),
      })
   }
}
