use crate::bytecode::Chunk;
use crate::natives::NativeFn;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

// Submodules
mod indexing;
mod native_operations;

/// Represents a Hinton range object.
#[derive(Clone)]
pub struct RangeObject {
   pub min: i64,
   pub max: i64,
}

/// Represents a Hinton iterator object.
pub struct IterObject {
   pub iter: Box<Object>,
   pub index: usize,
}

/// Represents a Hinton function object.
#[derive(Clone)]
pub struct FuncObject {
   pub defaults: Vec<Object>,
   pub min_arity: u8,
   pub max_arity: u8,
   pub chunk: Chunk,
   pub name: String,
   pub up_val_count: usize,
}

impl Default for FuncObject {
   fn default() -> Self {
      Self {
         defaults: vec![],
         min_arity: 0,
         max_arity: 0,
         chunk: Chunk::new(),
         name: String::from(""),
         up_val_count: 0,
      }
   }
}

/// Represents a Hinton native function object.
#[derive(Clone)]
pub struct NativeFuncObj {
   pub name: String,
   pub min_arity: u8,
   pub max_arity: u8,
   pub body: NativeFn,
}

/// Represents a Hinton closure object.
#[derive(Clone)]
pub struct ClosureObject {
   pub function: Rc<RefCell<FuncObject>>,
   pub up_values: Vec<Rc<RefCell<UpValRef>>>,
}

/// Represents a closure UpValue reference.
#[derive(Clone)]
pub enum UpValRef {
   Open(usize),
   Closed(Object),
}

impl UpValRef {
   /// Checks that the UpValue is open and points to the given stack index.
   pub fn is_open_at(&self, index: usize) -> bool {
      match self {
         UpValRef::Closed(_) => false,
         UpValRef::Open(i) => *i == index,
      }
   }
}

/// Represents a Hinton class object.
#[derive(Clone)]
pub struct ClassObject {
   pub name: String,
   pub members: Rc<RefCell<HashMap<String, Object>>>,
}

/// Represents a Hinton Instance object.
#[derive(Clone)]
pub struct InstanceObject {
   pub class: ClassObject,
   pub members: HashMap<String, Object>,
}

/// Represents a Hinton bound method.
#[derive(Clone)]
pub struct BoundMethod {
   pub receiver: Rc<RefCell<InstanceObject>>,
   pub method: ClosureObject,
}

/// All types of objects in Hinton
#[derive(Clone)]
pub enum Object {
   Bool(bool),
   Float(f64),
   Int(i64),
   Null,
   Range(RangeObject),
   Class(ClassObject),

   // Heap-allocated
   Array(Rc<RefCell<Vec<Object>>>),
   BoundMethod(BoundMethod),
   Closure(ClosureObject),
   Dict(Rc<RefCell<HashMap<String, Object>>>),
   Function(Rc<RefCell<FuncObject>>),
   Instance(Rc<RefCell<InstanceObject>>),
   Iter(Rc<RefCell<IterObject>>),
   Native(Box<NativeFuncObj>),
   String(String),
   Tuple(Vec<Object>),
}

impl Object {
   /// Gets the string type name of this object.
   pub fn type_name(&self) -> String {
      return match self {
         Self::Array(_) => String::from("Array"),
         Self::Bool(_) => String::from("Bool"),
         Self::Dict(_) => String::from("Dict"),
         Self::Float(_) => String::from("Float"),
         Self::Function(_) | Self::Native(_) | Self::Closure(_) | Self::BoundMethod(_) => {
            String::from("Function")
         }
         Self::Int(_) => String::from("Int"),
         Self::Iter(_) => String::from("Iter"),
         Self::Null => String::from("Null"),
         Self::Range(_) => String::from("Range"),
         Self::String(_) => String::from("String"),
         Self::Tuple(_) => String::from("Tuple"),
         Self::Class(c) => c.name.clone(),
         Self::Instance(i) => i.borrow().class.name.clone(),
      };
   }

   /// Checks that this object is a Hinton integer.
   pub fn is_int(&self) -> bool {
      matches!(self, Object::Int(_))
   }

   /// Checks that this object is a Hinton float.
   pub fn is_float(&self) -> bool {
      matches!(self, Object::Float(_))
   }

   /// Checks that this object is a Hinton boolean.
   pub fn is_bool(&self) -> bool {
      matches!(self, Object::Bool(_))
   }

   /// Checks that this object is a Hinton null.
   pub fn is_null(&self) -> bool {
      matches!(self, Object::Null)
   }

   /// Checks that this object is falsey.
   pub fn is_falsey(&self) -> bool {
      match self {
         Object::Null => true,
         Object::Bool(val) => !val,
         Object::Int(x) if *x == 0i64 => true,
         Object::Float(x) if *x == 0f64 => true,
         _ => false,
      }
   }

   /// Tries to convert this object to a Rust i64 integer.
   pub fn as_int(&self) -> Option<i64> {
      match self {
         Object::Int(v) => Some(*v),
         Object::Bool(b) => {
            if *b {
               Some(1i64)
            } else {
               Some(0i64)
            }
         }
         _ => None,
      }
   }

   /// Tries to convert this object to a Rust f64 float.
   pub fn as_float(&self) -> Option<f64> {
      match self {
         Object::Float(v) => Some(*v),
         _ => None,
      }
   }

   /// Tries to convert this object to a Rust string.
   pub fn as_string(&self) -> Option<&String> {
      match self {
         Object::String(s) => Some(s),
         _ => None,
      }
   }

   /// Tries to convert this object to a Rust boolean.
   pub fn as_bool(&self) -> Option<bool> {
      match self {
         Object::Bool(v) => Some(*v),
         _ => None,
      }
   }

   /// Tries to convert this object to a Hinton range object.
   pub fn as_range(&self) -> Option<&RangeObject> {
      match self {
         Object::Range(v) => Some(v),
         _ => None,
      }
   }

   /// Tries to convert this object to a Rust vector of Hinton objects (array).
   pub fn as_array(&self) -> Option<&Rc<RefCell<Vec<Object>>>> {
      match self {
         Object::Array(v) => Some(v),
         _ => None,
      }
   }

   /// Tries to convert this object to a Rust vector of Hinton objects (tuple).
   pub fn as_tuple(&self) -> Option<&Vec<Object>> {
      match self {
         Object::Tuple(v) => Some(v),
         _ => None,
      }
   }

   /// Tries to convert this object to a Hinton function object (only used for bytecode pretty
   /// printing.
   #[cfg(feature = "show_bytecode")]
   pub fn as_function(&self) -> Option<&Rc<RefCell<FuncObject>>> {
      match self {
         Object::Function(v) => Some(v),
         _ => None,
      }
   }

   /// Checks that this object is equal to the provided object according to Hinton's rules for
   /// object equality.
   pub fn equals(&self, right: &Object) -> bool {
      // Equality check for number-like types
      match self {
         Object::Int(i) => {
            return match right {
               Object::Int(x) if i == x => true,
               Object::Float(x) if (x - *i as f64) == 0f64 => true,
               Object::Bool(x) if (i == &0i64 && !*x) || (i == &1i64 && *x) => true,
               _ => false,
            }
         }
         Object::Float(f) => {
            return match right {
               Object::Int(x) if (f - *x as f64) == 0f64 => true,
               Object::Float(x) if f == x => true,
               Object::Bool(x) if (f == &0f64 && !*x) || (f == &1f64 && *x) => true,
               _ => false,
            }
         }
         Object::Bool(b) => {
            return match right {
               Object::Int(x) if (x == &0i64 && !*b) || (x == &1i64 && *b) => true,
               Object::Float(x) if (x == &0f64 && !*b) || (x == &1f64 && *b) => true,
               Object::Bool(x) => !(b ^ x),
               _ => false,
            }
         }
         _ => {}
      }

      // If the operands differ in type, we can safely assume
      // they are not equal in value.
      if std::mem::discriminant(self) != std::mem::discriminant(&right) {
         return false;
      }

      // At this point, the operands have the same type, so we
      // proceed to check if they match in value.
      return match self {
         Object::String(a) => (a == right.as_string().unwrap()),
         Object::Array(a) => {
            let a = &a.borrow();
            let b = &right.as_array().unwrap().borrow();

            // If the arrays differ in size, they must differ in value. However, if they
            // are equal in size, then we must check that each item match.
            if a.len() != b.len() {
               false
            } else {
               for i in 0..a.len() {
                  // If at least one of the items differ in value,
                  // then the arrays are not equals.
                  if !&a[i].equals(&b[i]) {
                     return false;
                  }
               }

               true
            }
         }
         Object::Tuple(a) => {
            let b = &right.as_tuple().unwrap();

            // If the tuples differ in size, they must differ in value. However, if they
            // are equal in size, then we must check that each item match.
            if a.len() != b.len() {
               false
            } else {
               for i in 0..a.len() {
                  // If at least one of the items differ in value,
                  // then the arrays are not equals.
                  if !&a[i].equals(&b[i]) {
                     return false;
                  }
               }

               true
            }
         }
         Object::Range(a) => {
            let b = right.as_range().unwrap();

            // If the ranges match in boundaries,
            // then they are equal in value.
            a.min == b.min && a.max == b.max
         }
         Object::Null => return matches!(right, Object::Null),
         _ => false,
      };
   }
}

/// Implements the `Display` trait so that objects can be printed in a console-friendly way.
impl<'a> fmt::Display for Object {
   fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
      match *self {
         Object::Int(ref inner) => {
            let str =
               String::from("\x1b[38;5;81m") + inner.to_string().as_str() + String::from("\x1b[0m").as_str();
            fmt::Display::fmt(&str, f)
         }
         Object::Float(ref inner) => {
            let str = String::from("\x1b[38;5;81m")
                    + inner.to_string().as_str()
                    + if inner.fract() == 0.0 { ".0" } else { "" } // display the .0
                    + String::from("\x1b[0m").as_str();
            fmt::Display::fmt(&str, f)
         }
         Object::String(ref inner) => fmt::Display::fmt(inner.as_str(), f),
         Object::Bool(inner) => {
            let str = if inner {
               String::from("\x1b[38;5;3mtrue\x1b[0m")
            } else {
               String::from("\x1b[38;5;3mfalse\x1b[0m")
            };

            fmt::Display::fmt(&str, f)
         }
         Object::Array(ref inner) => {
            let arr = &inner.borrow();

            let mut arr_str = String::from("[");
            for (idx, obj) in arr.iter().enumerate() {
               if idx == arr.len() - 1 {
                  arr_str += &(format!("{}", obj))[..]
               } else {
                  arr_str += &(format!("{}, ", obj))[..];
               }
            }
            arr_str += "]";

            write!(f, "{}", arr_str)
         }
         Object::Tuple(ref inner) => {
            let mut arr_str = String::from("(");
            for (idx, obj) in inner.iter().enumerate() {
               if idx == inner.len() - 1 {
                  arr_str += &(format!("{}", obj))[..]
               } else {
                  arr_str += &(format!("{}, ", obj))[..];
               }
            }
            arr_str += ")";

            write!(f, "{}", arr_str)
         }
         Object::Range(ref inner) => {
            write!(
               f,
               "[\x1b[38;5;81m{}\x1b[0m..\x1b[38;5;81m{}\x1b[0m]",
               inner.min, inner.max
            )
         }
         Object::Iter(ref inner) => {
            let str = format!("<Iterable '{}'>", inner.borrow_mut().iter.type_name());
            fmt::Display::fmt(&str, f)
         }
         Object::Function(ref inner) => {
            let name = &inner.borrow_mut().name;

            let str = if name.is_empty() {
               String::from("<script>")
            } else {
               format!("<Func '{}'>", name)
            };

            fmt::Display::fmt(&str, f)
         }
         Object::Closure(ref inner) => {
            let name = &inner.function.borrow().name;

            let str = if name.is_empty() {
               String::from("<Func script>")
            } else {
               format!("<Func '{}'>", name)
            };

            fmt::Display::fmt(&str, f)
         }
         Object::BoundMethod(ref inner) => {
            let name = &inner.method.function.borrow().name;

            let str = if name.is_empty() {
               String::from("<Func script>")
            } else {
               format!("<Func '{}'>", name)
            };

            fmt::Display::fmt(&str, f)
         }
         Object::Class(ref inner) => fmt::Display::fmt(&format!("<Class '{}'>", inner.name), f),
         Object::Instance(ref inner) => {
            fmt::Display::fmt(&format!("<Instance of '{}'>", inner.borrow().class.name), f)
         }
         Object::Native(ref inner) => {
            let str = format!("<NativeFn '{}'>", inner.name);
            fmt::Display::fmt(&str, f)
         }
         Object::Dict(ref inner) => {
            let mut arr_str = String::from("{");

            for (idx, key) in inner.borrow().keys().enumerate() {
               if idx == inner.borrow().keys().len() - 1 {
                  arr_str += &(format!("'{}': {}", key, inner.borrow().get(key).unwrap()))[..]
               } else {
                  arr_str += &(format!("'{}': {}, ", key, inner.borrow().get(key).unwrap()))[..]
               }
            }

            arr_str += "}";

            write!(f, "{}", arr_str)
         }
         Object::Null => f.write_str("\x1b[37;1mnull\x1b[0m"),
      }
   }
}
