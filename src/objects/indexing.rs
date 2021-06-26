use crate::errors::ObjectOprErrType;
use crate::objects::{Object, RangeObject};

impl Object {
   /// Defines the indexing operation of Hinton objects.
   pub fn subscript(&self, index: &Object) -> Result<Object, ObjectOprErrType> {
      match self {
         Object::Array(arr) => index_array(&arr.borrow(), index),
         Object::Tuple(tup) => index_tuple(&tup, index),
         Object::String(str) => index_string(&str, index),
         Object::Range(range) => index_range(range, index),
         _ => {
            return Err(ObjectOprErrType::TypeError(format!(
               "Cannot index object of type '{}'.",
               self.type_name()
            )))
         }
      }
   }
}

/// Takes an i64 integer and converts it into an object index. This allows indexing objects with
/// negative integers.
///
/// # Parameters
/// - `x`: The positive or negative index.
/// - `len`: The length of the object.
///
/// # Returns
/// - `Option<usize>`: Return Some(usize) if the index is within the bounds of the object's length
/// or `None` otherwise.
fn to_bounded_index(x: &i64, len: usize) -> Option<usize> {
   if x >= &0 && (*x as usize) < len {
      Some(*x as usize)
   } else if x < &0 && (i64::abs(*x) as usize <= len) {
      Some(len - i64::abs(*x) as usize)
   } else {
      None
   }
}

/// Get the ith object in a Hinton array.
///
/// # Parameters
/// - `arr`: A reference to the underlying `Vec<Object>` in a Hinton Array.
/// - `index`: A Hinton object that will serve as the index of the array. For example, this object
/// could be a Hinton integer, or a Hinton range.
///
/// # Returns
/// - `Result<Object, ObjectOprErrType>`: Returns `Ok(Object)` with a Hinton Object if the index is
/// within bounds. Returns `Err(ObjectOprErrType)` if there was an error while indexing the array.
fn index_array(arr: &[Object], index: &Object) -> Result<Object, ObjectOprErrType> {
   match index {
      // Indexing type: Array[Int]
      Object::Int(idx) => {
         if let Some(pos) = to_bounded_index(idx, arr.len()) {
            if let Some(val) = arr.get(pos) {
               return Ok(val.clone());
            }
         }
      }
      // Indexing type: Array[Bool]
      Object::Bool(val) => {
         let pos = (if *val { 1 } else { 0 }) as usize;
         if let Some(val) = arr.get(pos) {
            return Ok(val.clone());
         }
      }
      // Indexing type: Array[Range]
      Object::Range(_) => {
         unimplemented!("Array indexing with ranges.")
      }
      _ => {
         return Err(ObjectOprErrType::TypeError(format!(
            "Array index must be an Int or a Range. Found '{}' instead.",
            index.type_name()
         )))
      }
   }
   Err(ObjectOprErrType::IndexError(String::from(
      "Array index out of bounds.",
   )))
}

/// Get the ith object in a Hinton tuple.
///
/// # Parameters
/// - `tup`: A reference to the underlying `Vec<Object>` in a Hinton tuple.
/// - `index`: A Hinton object that will serve as the index of the tuple. For example, this object
/// could be a Hinton integer, or a Hinton range.
///
/// # Returns
/// - `Result<Object, ObjectOprErrType>`: Returns `Ok(Object)` with a Hinton Object if the index is
/// within bounds. Returns `Err(ObjectOprErrType)` if there was an error while indexing the tuple.
fn index_tuple(tup: &[Object], index: &Object) -> Result<Object, ObjectOprErrType> {
   match index {
      // Indexing type: Tuple[Int]
      Object::Int(idx) => {
         if let Some(pos) = to_bounded_index(idx, tup.len()) {
            if let Some(val) = tup.get(pos) {
               return Ok(val.clone());
            }
         }
      }
      // Indexing type: Tuple[Bool]
      Object::Bool(val) => {
         let pos = (if *val { 1 } else { 0 }) as usize;

         if let Some(val) = tup.get(pos) {
            return Ok(val.clone());
         }
      }
      // Indexing type: Tuple[Range]
      Object::Range(_) => {
         unimplemented!("Tuple indexing with ranges.")
      }
      _ => {
         return Err(ObjectOprErrType::TypeError(format!(
            "Tuple index must be an Int or a Range. Found '{}' instead.",
            index.type_name()
         )))
      }
   }

   Err(ObjectOprErrType::IndexError(String::from(
      "Tuple index out of bounds.",
   )))
}

/// Get the ith character in a Hinton string.
///
/// # Parameters
/// - `str`: A reference to the underlying `String` in a Hinton string.
/// - `index`: A Hinton object that will serve as the index of the string. For example, this object
/// could be a Hinton integer, or a Hinton range.
///
/// # Returns
/// - `Result<Object, ObjectOprErrType>`: Returns `Ok(Object)` with a Hinton Object if the index is
/// within bounds. Returns `Err(ObjectOprErrType)` if there was an error while indexing the string.
fn index_string(str: &str, index: &Object) -> Result<Object, ObjectOprErrType> {
   match index {
      // Indexing type: String[Int]
      Object::Int(idx) => {
         let chars: Vec<char> = str.chars().collect();

         if let Some(pos) = to_bounded_index(idx, chars.len()) {
            if let Some(val) = chars.get(pos) {
               return Ok(Object::String(val.to_string()));
            }
         }
      }
      // Indexing type: String[Bool]
      Object::Bool(val) => {
         let chars: Vec<char> = str.chars().collect();
         let pos = (if *val { 1 } else { 0 }) as usize;

         if let Some(val) = chars.get(pos) {
            return Ok(Object::String(val.to_string()));
         }
      }
      // Indexing type: String[Range]
      Object::Range(_) => {
         unimplemented!("String indexing with ranges.")
      }
      _ => {
         return Err(ObjectOprErrType::TypeError(format!(
            "String index must be an Int or a Range. Found '{}' instead.",
            index.type_name()
         )))
      }
   }

   Err(ObjectOprErrType::IndexError(String::from(
      "String index out of bounds.",
   )))
}

/// Get the ith object in a Hinton range.
///
/// # Parameters
/// - `range`: A reference to the underlying `RangeObject` in a Hinton range.
/// - `index`: A Hinton object that will serve as the index of the range. For example, this object
/// could be a Hinton integer, or a Hinton range.
///
/// # Returns
/// - `Result<Object, ObjectOprErrType>`: Returns `Ok(Object)` with a Hinton Object if the index is
/// within bounds. Returns `Err(ObjectOprErrType)` if there was an error while indexing the range.
fn index_range(range: &RangeObject, index: &Object) -> Result<Object, ObjectOprErrType> {
   match index {
      // Indexing type: Range[Int]
      Object::Int(idx) => {
         let min = range.min;
         let max = range.max;

         if let Some(pos) = to_bounded_index(idx, i64::abs(max - min) as usize) {
            return if max - min > 0 {
               Ok(Object::Int(min + pos as i64))
            } else {
               Ok(Object::Int(min - pos as i64))
            };
         }
      }
      // Indexing type: Range[Bool]
      Object::Bool(val) => {
         let idx = (if *val { 1 } else { 0 }) as i64;
         let min = range.min;
         let max = range.max;

         if let Some(pos) = to_bounded_index(&idx, i64::abs(max - min) as usize) {
            return if max - min > 0 {
               Ok(Object::Int(min + pos as i64))
            } else {
               Ok(Object::Int(min - pos as i64))
            };
         }
      }
      // Indexing type: Range[Range]
      Object::Range(_) => {
         unimplemented!("Range indexing with ranges.")
      }
      _ => {
         return Err(ObjectOprErrType::TypeError(format!(
            "Range index must be an Int or a Range. Found '{}' instead.",
            index.type_name()
         )))
      }
   }

   Err(ObjectOprErrType::IndexError(String::from(
      "Range index out of bounds.",
   )))
}
