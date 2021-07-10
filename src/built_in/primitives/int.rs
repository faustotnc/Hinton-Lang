use crate::built_in::primitives::HTPrimitive;
use crate::built_in::NativeBoundMethod;
use crate::errors::RuntimeErrorType;
use crate::objects::class_obj::{ClassField, ClassObject};
use crate::objects::Object;
use crate::virtual_machine::{RuntimeResult, VM};
use hashbrown::HashMap;

/// Represents the Hinton `Int` primitive class.
pub struct IntClass(ClassObject);

/// Implements the `HTPrimitive` trait for the `IntClass` primitive Hinton class.
impl HTPrimitive for IntClass {
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
      let mut _self = IntClass(ClassObject::new("Int"));

      // >>>>>>> Class fields for the "Int" primitive type to be added after this line
      _self.bind_field("BITS", true, true, false, true, Object::Int(i64::BITS as i64));
      _self.bind_field("MAX", true, true, false, true, Object::Int(i64::MAX));
      _self.bind_field("MIN", true, true, false, true, Object::Int(i64::MIN));
      _self.bind_non_static_method("abs", (0, 0), abs as NativeBoundMethod);
      _self.bind_non_static_method("bit_len", (0, 0), bit_len as NativeBoundMethod);
      _self.bind_non_static_method("count_ones", (0, 0), count_ones as NativeBoundMethod);
      _self.bind_non_static_method("count_zeros", (0, 0), count_zeros as NativeBoundMethod);
      _self.bind_non_static_method("leading_ones", (0, 0), leading_ones as NativeBoundMethod);
      _self.bind_non_static_method("leading_zeros", (0, 0), leading_zeros as NativeBoundMethod);
      _self.bind_non_static_method("to_string", (0, 0), to_string as NativeBoundMethod);
      _self.bind_non_static_method("trailing_ones", (0, 0), trailing_ones as NativeBoundMethod);
      _self.bind_non_static_method("trailing_zeros", (0, 0), trailing_zeros as NativeBoundMethod);
      // <<<<<<< Class fields for the "Int" primitive type to be added before this line

      _self.0
   }
}

macro_rules! verify_int_object {
   ($o: expr, $prop_name: expr) => {
      match $o {
         Object::Int(i) => i,
         _ => {
            return RuntimeResult::Error {
               error: RuntimeErrorType::TypeError,
               message: format!(
                  "Property 'Int.{}' requires that 'self' be an Int. Found '{}' instead.",
                  $prop_name,
                  $o.type_name()
               ),
            }
         }
      }
   };
}

/// Converts a Hinton integer into a Hinton string.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `this`: The integer object.
/// * `_`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn to_string(vm: &mut VM, this: Object, _: Vec<Object>) -> RuntimeResult {
   vm.push_stack(Object::from(format!("{}", verify_int_object!(this, "to_string"))))
}

/// Counts the number of zeros in the binary representation of this Hinton integer.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `this`: The integer object.
/// * `_`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn count_zeros(vm: &mut VM, this: Object, _: Vec<Object>) -> RuntimeResult {
   vm.push_stack(Object::Int(
      verify_int_object!(this, "count_zeros").count_zeros() as i64
   ))
}

/// Counts the number of ones in the binary representation of this Hinton integer.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `this`: The integer object.
/// * `_`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn count_ones(vm: &mut VM, this: Object, _: Vec<Object>) -> RuntimeResult {
   vm.push_stack(Object::Int(
      verify_int_object!(this, "count_ones").count_ones() as i64
   ))
}

/// Counts the number of leading zeros in the binary representation of this Hinton integer.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `this`: The integer object.
/// * `_`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn leading_zeros(vm: &mut VM, this: Object, _: Vec<Object>) -> RuntimeResult {
   vm.push_stack(Object::Int(
      verify_int_object!(this, "leading_zeros").leading_zeros() as i64,
   ))
}

/// Counts the number of trailing zeros in the binary representation of this Hinton integer.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `this`: The integer object.
/// * `_`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn trailing_zeros(vm: &mut VM, this: Object, _: Vec<Object>) -> RuntimeResult {
   vm.push_stack(Object::Int(
      verify_int_object!(this, "trailing_zeros").trailing_zeros() as i64,
   ))
}

/// Counts the number of leading ones in the binary representation of this Hinton integer.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `this`: The integer object.
/// * `_`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn leading_ones(vm: &mut VM, this: Object, _: Vec<Object>) -> RuntimeResult {
   vm.push_stack(Object::Int(
      verify_int_object!(this, "leading_ones").leading_ones() as i64,
   ))
}

/// Counts the number of trailing ones in the binary representation of this Hinton integer.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `this`: The integer object.
/// * `_`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn trailing_ones(vm: &mut VM, this: Object, _: Vec<Object>) -> RuntimeResult {
   vm.push_stack(Object::Int(
      verify_int_object!(this, "trailing_ones").trailing_ones() as i64,
   ))
}

/// Computes the absolute value of this Hinton integer.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `this`: The integer object.
/// * `_`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn abs(vm: &mut VM, this: Object, _: Vec<Object>) -> RuntimeResult {
   vm.push_stack(Object::Int(verify_int_object!(this, "abs").abs()))
}

/// Computes the number of bits needed to represent this Hinton integer in binary form.
///
/// # Arguments
/// * `vm`: A mutable reference to the virtual machine.
/// * `this`: The integer object.
/// * `_`: A vector of objects that will serve as arguments to this method call.
///
/// # Returns:
/// RuntimeResult
fn bit_len(vm: &mut VM, this: Object, _: Vec<Object>) -> RuntimeResult {
   let i = verify_int_object!(this, "bit_len");
   vm.push_stack(Object::Int((i.abs() as f64).log2().ceil() as i64))
}
