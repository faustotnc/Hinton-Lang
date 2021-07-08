use crate::built_in::primitives::HTPrimitive;
use crate::built_in::NativeBoundMethod;
use crate::errors::RuntimeErrorType;
use crate::objects::{ClassObject, Object};
use crate::virtual_machine::{RuntimeResult, VM};
use hashbrown::HashMap;

/// Represents the Hinton `Int` primitive class.
pub type IntClass = ClassObject;

/// Implements the `HTPrimitive` trait for the `IntClass` primitive Hinton class.
impl HTPrimitive for IntClass {
   fn name(&self) -> String {
      self.name.clone()
   }

   fn members(&mut self) -> &mut HashMap<String, Object> {
      &mut self.members
   }
}

/// The default implementation of a Hinton primitive `Int` class.
impl Default for IntClass {
   fn default() -> Self {
      let mut _self = ClassObject::new("Int");

      // >>>>>>> Native methods for the "Int" primitive type to be added after this line
      _self.bind_method("to_string", (0, 0), int_to_string as NativeBoundMethod);
      // <<<<<<< Native methods for the "Int" primitive type to be added before this line

      _self
   }
}

macro_rules! verify_int_object {
   ($o: expr, $fn_name: expr) => {
      match $o {
         Object::Int(i) => i,
         _ => {
            return RuntimeResult::Error {
               error: RuntimeErrorType::TypeError,
               message: format!(
                  "Method 'Int.{}' requires that 'self' be an Int. Found '{}' instead.",
                  $fn_name,
                  $o.type_name()
               ),
            }
         }
      }
   };
}

/// Hinton primitive bound method for converting an integer to a string.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `this`: The integer object to be converted to a string.
/// * `_`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn int_to_string(vm: &mut VM, this: Object, _: Vec<Object>) -> RuntimeResult {
   let i = verify_int_object!(this, "to_string");
   vm.push_stack(Object::from(format!("{}", i)))
}
