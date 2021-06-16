use crate::{
    errors::RuntimeErrorType,
    objects::{IterObject, NativeFuncObj, Object},
    virtual_machine::RuntimeResult,
};
use std::{borrow::Borrow, cell::RefCell, io, rc::Rc, time::SystemTime};

/// Represents the body of a Hinton native function object.
pub type NativeFn = fn(Vec<Object>) -> Result<Object, RuntimeResult>;

/// Gets a native function by name.
///
/// ## Arguments
/// * `name` – The name of the native function.
///
/// ## Returns
/// Returns a native function if the provided name matches a native function's name,
/// otherwise returns a RuntimeResult error.
pub fn get_native_fn(name: &str) -> Result<NativeFuncObj, RuntimeResult> {
    match name {
        "print" => Ok(NativeFuncObj {
            name: String::from("print"),
            min_arity: 1,
            max_arity: 1,
            function: native_print as NativeFn,
        }),
        "input" => Ok(NativeFuncObj {
            name: String::from("input"),
            min_arity: 1,
            max_arity: 1,
            function: native_input as NativeFn,
        }),
        "iter" => Ok(NativeFuncObj {
            name: String::from("iter"),
            min_arity: 1,
            max_arity: 1,
            function: native_iter as NativeFn,
        }),
        "next" => Ok(NativeFuncObj {
            name: String::from("next"),
            min_arity: 1,
            max_arity: 1,
            function: native_next as NativeFn,
        }),
        "clock" => Ok(NativeFuncObj {
            name: String::from("clock"),
            min_arity: 0,
            max_arity: 0,
            function: native_clock as NativeFn,
        }),
        "assert" => Ok(NativeFuncObj {
            name: String::from("assert"),
            min_arity: 1,
            max_arity: 2,
            function: native_assert as NativeFn,
        }),
        "assert_eq" => Ok(NativeFuncObj {
            name: String::from("assert_eq"),
            min_arity: 2,
            max_arity: 3,
            function: native_assert_eq as NativeFn,
        }),
        "assert_ne" => Ok(NativeFuncObj {
            name: String::from("assert_ne"),
            min_arity: 2,
            max_arity: 3,
            function: native_assert_ne as NativeFn,
        }),
        _ => Err(RuntimeResult::Error {
            error: RuntimeErrorType::ReferenceError,
            message: format!("No native function named '{}'.", name),
        }),
    }
}

/// Finds and executes a native function by name.
///
/// ## Arguments
/// * `name` – The name of the native function.
/// * `args` – A vector of objects (the arguments) for the native function.
///
/// ## Returns
/// Returns the result of the call if the function was executed successfully, otherwise
/// returns a RuntimeResult error.
pub fn call_native(name: &str, args: Vec<Object>) -> Result<Object, RuntimeResult> {
    match get_native_fn(name) {
        Ok(f) => {
            let args_len = args.len() as u8;

            // Checks the argument arity for the function call.
            if args_len < f.min_arity || args_len > f.max_arity {
                if f.min_arity == f.max_arity {
                    return Err(RuntimeResult::Error {
                        error: RuntimeErrorType::ArgumentError,
                        message: format!("Expected {} arguments but got {} instead.", f.min_arity, args_len),
                    });
                } else {
                    return Err(RuntimeResult::Error {
                        error: RuntimeErrorType::ArgumentError,
                        message: format!(
                            "Expected {} to {} arguments but got {} instead.",
                            f.min_arity, f.max_arity, args_len
                        ),
                    });
                }
            }

            // Calls the native function
            let call_result = (f.function)(args);

            // Returns the result of the call
            return call_result;
        }
        Err(e) => Err(e),
    }
}

/// Checks whether or not a string name is associated with a native function.
///
/// ## Arguments
/// * `name` – The name of the native function.
///
/// ## Returns
/// True if the name maps to a native function. False otherwise.
pub fn check_is_native(name: &str) -> bool {
    match get_native_fn(name) {
        Ok(_) => true,
        Err(_) => false,
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
    match args[0].borrow() {
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
    let obj = match iter.iter.get_at_index(&current_index) {
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

/// Checks if a Hinton iterator has a next item.
pub fn iter_has_next(o: &Rc<RefCell<IterObject>>) -> bool {
    let o = o.borrow_mut();

    let len = match o.iter.borrow() {
        Object::String(ref x) => x.borrow_mut().len(),
        Object::Tuple(ref x) => x.tup.len(),
        Object::Range(ref x) => i64::abs(x.max - x.min) as usize,
        Object::Array(ref x) => {
            let a = &x.borrow_mut();
            a.len()
        }
        _ => unreachable!("Object is not iterable."),
    };

    o.index < len
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
                    Ok(Object::String(Rc::new(RefCell::new(input))))
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
            let str = Rc::new(RefCell::new(String::from("Assertion failed on a falsey value.")));
            Object::String(str)
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
            let str = Rc::new(RefCell::new(String::from("Assertion values are not equal.")));
            Object::String(str)
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
            let str = Rc::new(RefCell::new(String::from("Assertion values are equal.")));
            Object::String(str)
        };

        Err(RuntimeResult::Error {
            error: RuntimeErrorType::AssertionError,
            message: format!("{}", message),
        })
    }
}
