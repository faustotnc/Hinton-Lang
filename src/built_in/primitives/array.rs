use crate::built_in::primitives::HTPrimitive;
use crate::built_in::NativeBoundMethod;
use crate::errors::RuntimeErrorType;
use crate::objects::class_obj::{ClassField, ClassObject};
use crate::objects::Object;
use crate::virtual_machine::{RuntimeResult, VM};
use hashbrown::HashMap;

/// Represents the Hinton `Array` primitive class.
pub struct ArrayClass(ClassObject);

/// Implements the `HTPrimitive` trait for the `ArrayClass` primitive Hinton class.
impl HTPrimitive for ArrayClass {
   /// Gets the name class name of this Hinton primitive.
   fn name(&self) -> String {
      self.0.name.clone()
   }

   /// Gets the non-static members of this Hinton primitive.
   fn members(&mut self) -> &mut HashMap<String, ClassField> {
      &mut self.0.members
   }

   /// Gets the static members of this Hinton primitive.
   fn statics(&mut self) -> &mut HashMap<String, ClassField> {
      &mut self.0.statics
   }

   /// Gets the default class object stored in this Hinton primitive.
   fn default() -> ClassObject {
      let mut _self = ArrayClass(ClassObject::new("Array"));

      // >>>>>>> Class fields for the "Array" primitive type to be added after this line
      _self.bind_non_static_method("index_of", (1, 1), index_of as NativeBoundMethod);
      _self.bind_non_static_method("len", (0, 0), len as NativeBoundMethod);
      _self.bind_non_static_method("pop", (0, 0), pop as NativeBoundMethod);
      _self.bind_non_static_method("push", (1, 1), push as NativeBoundMethod);
      // <<<<<<< Class fields for the "Array" primitive type to be added before this line

      _self.0
   }
}

macro_rules! verify_array_object {
   ($maybe_array: expr, $prop_name: expr) => {
      match $maybe_array {
         Object::Array(a) => a,
         _ => {
            return RuntimeResult::Error {
               error: RuntimeErrorType::TypeError,
               message: format!(
                  "Property 'Array.{}' requires that 'self' be an Array. Found '{}' instead.",
                  $prop_name,
                  $maybe_array.type_name()
               ),
            }
         }
      }
   };
}

/// Gets the length of a Hinton array.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `this`: The array object.
/// * `_`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn len(vm: &mut VM, this: Object, _: Vec<Object>) -> RuntimeResult {
   vm.push_stack(Object::from(verify_array_object!(this, "len").borrow().len()))
}

/// Pushes an object into this Hinton array.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `this`: The array object.
/// * `args`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn push(vm: &mut VM, this: Object, args: Vec<Object>) -> RuntimeResult {
   let obj = args[0].clone();
   verify_array_object!(this, "push").borrow_mut().push(obj);

   vm.push_stack(Object::Null)
}

/// Pops an object from this Hinton array.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `this`: The array object.
/// * `_`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn pop(vm: &mut VM, this: Object, _: Vec<Object>) -> RuntimeResult {
   match verify_array_object!(this, "pop").borrow_mut().pop() {
      Some(o) => vm.push_stack(o),
      None => RuntimeResult::Error {
         error: RuntimeErrorType::IndexError,
         message: "Attempted to pop from an empty array.".to_string(),
      },
   }
}

/// Gets the index of the first occurrence of an object in this Hinton array.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `this`: The array object.
/// * `args`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn index_of(vm: &mut VM, this: Object, args: Vec<Object>) -> RuntimeResult {
   let obj = args[0].clone();

   match verify_array_object!(this, "index_of")
      .borrow_mut()
      .iter()
      .position(|x| x == &obj)
   {
      Some(i) => vm.push_stack(Object::Int(i as i64)),
      None => vm.push_stack(Object::Null),
   }
}
