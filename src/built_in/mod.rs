use crate::built_in::natives::Natives;
use crate::built_in::primitives::Primitives;
use crate::objects::{NativeFuncObj, NativeMethodObj, Object};
use crate::virtual_machine::{RuntimeResult, VM};

// Submodules
pub mod natives;
pub mod primitives;

/// Represents the body of a Hinton native function object.
pub type NativeFn = fn(&mut VM, Vec<Object>) -> RuntimeResult;

/// Represents the body of a Hinton primitive-bound method object.
pub type NativeBoundMethod = fn(&mut VM, Object, Vec<Object>) -> RuntimeResult;

/// Represents a collection of built-in functions and classes available
/// by default in any Hinton program.
pub struct BuiltIn {
   pub natives: Natives,
   pub primitives: Primitives,
}

/// The default implementation for the `BuiltIn` struct.
impl Default for BuiltIn {
   fn default() -> Self {
      Self {
         natives: Natives::default(),
         primitives: Primitives::default(),
      }
   }
}

impl BuiltIn {
   /// Gets a property attached to a Hinton primitive class.
   ///
   /// # Arguments
   /// * `vm`: A mutable reference to the Virtual machine.
   /// * `val`: The value associated with the primitive.
   /// * `name`: The class name of the primitive.
   /// * `prop`: The name of the property to get from the primitive.
   ///
   /// # Returns
   /// RuntimeResult
   ///
   /// # Examples
   /// ```
   /// BuiltIn::primitive_prop(&mut vm, Object::Int(33), "to_string", String::from("Int"))
   /// ```
   pub fn primitive_prop(vm: &mut VM, val: Object, name: &str, prop: String) -> RuntimeResult {
      match vm.built_in.primitives.get_prop_in_class(name, prop) {
         Ok(o) => vm.push_stack(match o {
            Object::BoundNativeMethod(b) => Object::from(NativeMethodObj {
               value: Box::new(val),
               ..b
            }),
            _ => val,
         }),
         Err(e) => e,
      }
   }

   /// Executes the body of a native function object.
   ///
   /// # Arguments
   /// * `vm`: A mutable reference to the Virtual machine.
   /// * `func`: The native function object to execute.
   /// * `args`: A vector of objects that will serve as the arguments to the function call.
   ///
   /// # Returns
   /// RuntimeResult
   ///
   /// # Examples
   /// ```
   /// let fn_print = NativeFuncObj {
   ///    name: "print".to_string(),
   ///    min_arity: 1,
   ///    max_arity: 1,
   ///    body: native_print as NativeFn,
   /// };
   ///
   /// BuiltIn::call_native_fn(&mut vm, fn_print, vec![Object::Bool(true)])
   /// ```
   pub fn call_native_fn(vm: &mut VM, func: NativeFuncObj, arg_count: u8) -> RuntimeResult {
      // Checks the argument arity for the function call.
      if let Err(e) = vm.arity_check(func.min_arity, func.max_arity, arg_count) {
         return e;
      }

      let mut args: Vec<Object> = Vec::with_capacity(arg_count as usize);
      for _ in 0..arg_count {
         let val = vm.pop_stack();
         args.push(val);
      }
      args.reverse();

      // Pop native function object off the stack before calling it.
      vm.pop_stack();

      // Calls the native function, and returns its result.
      (func.body)(vm, args)
   }

   /// Executes the body of a method bound to primitive class.
   ///
   /// # Arguments
   /// * `vm`: A mutable reference to the virtual machine.
   /// * `func`: The native bound method to execute.
   /// * `args`: A vector of objects that will serve as the arguments to the method call.
   ///
   /// # Returns
   /// RuntimeResult
   ///
   /// # Examples
   /// ```
   /// let array_len_method = NativeMethodObj {
   ///    class_name: String::from("Array"),
   ///    method_name: String::from("len"),
   ///    value: Box::new(Object::Array(
   ///       Rc::new(RefCell::new(vec![Object::from(1), Object::from(2), Object::from(3)]))
   ///    )),
   ///    min_arity: 0,
   ///    max_arity: 0,
   ///    body: array_len as NativeBoundMethod,
   /// };
   ///
   /// BuiltIn::call_bound_method(&mut vm, array_len_method, vec![]);
   /// ```
   pub fn call_bound_method(vm: &mut VM, func: NativeMethodObj, arg_count: u8) -> RuntimeResult {
      // Checks the argument arity for the function call.
      if let Err(e) = vm.arity_check(func.min_arity, func.max_arity, arg_count) {
         return e;
      }

      let mut args: Vec<Object> = Vec::with_capacity(arg_count as usize);
      for _ in 0..arg_count {
         let val = vm.pop_stack();
         args.push(val);
      }
      args.reverse();

      // Pop native function object off the stack before calling it.
      vm.pop_stack();

      // Calls the native function, and returns its result.
      (func.body)(vm, *func.value, args)
   }
}
