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

/// Represents a Hinton class field.
#[derive(Clone)]
pub struct ClassFieldObject {
   pub value: Box<Object>,
   pub is_constant: bool,
}

/// All types of objects in Hinton
#[derive(Clone)]
pub enum Object {
   Array(Rc<RefCell<Vec<Object>>>),
   Bool(bool),
   BoundMethod(BoundMethod),
   Class(ClassObject),
   ClassField(ClassFieldObject),
   Closure(ClosureObject),
   Dict(Rc<RefCell<HashMap<String, Object>>>),
   Float(f64),
   Function(Rc<RefCell<FuncObject>>),
   Instance(Rc<RefCell<InstanceObject>>),
   Int(i64),
   Iter(Rc<RefCell<IterObject>>),
   Native(Box<NativeFuncObj>),
   Null,
   Range(RangeObject),
   String(String),
   Tuple(Vec<Object>),
}

/// Checks that two vectors of objects are equal in value.
///
/// # Parameters
/// - `v1`: The first vector of objects.
/// - `v2`: The second vector of objects.
///
/// # Returns
/// `bool`: True if the vectors are equal, false otherwise.
pub fn obj_vectors_equal(v1: &[Object], v2: &[Object]) -> bool {
   if v1.len() != v2.len() {
      false
   } else {
      for (i, o) in v1.iter().enumerate() {
         if !o.equals(&v2[i]) {
            return false;
         }
      }
      true
   }
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
         Self::ClassField(c) => c.value.type_name(),
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
         Self::Null => true,
         Self::Bool(val) => !val,
         Self::Int(x) if *x == 0i64 => true,
         Self::Float(x) if *x == 0f64 => true,
         Self::ClassField(c) => c.value.is_falsey(),
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

   /// Tries to convert this object to a Rust boolean.
   pub fn as_bool(&self) -> Option<bool> {
      match self {
         Object::Bool(v) => Some(*v),
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
      // If the rhs is a class field, we match `self` against the wrapped
      // value of the field, making recursive calls to `Object.equals(...)`
      // in case the value is itself also a class field.
      if let Object::ClassField(r) = right {
         return self.equals(&r.value);
      }

      match self {
         Object::Int(i) => match right {
            Object::Int(x) if i == x => true,
            Object::Float(x) if (x - *i as f64) == 0f64 => true,
            Object::Bool(x) if (i == &0i64 && !*x) || (i == &1i64 && *x) => true,
            _ => false,
         },
         Object::Float(f) => match right {
            Object::Int(x) if (f - *x as f64) == 0f64 => true,
            Object::Float(x) if f == x => true,
            Object::Bool(x) if (f == &0f64 && !*x) || (f == &1f64 && *x) => true,
            _ => false,
         },
         Object::Bool(b) => match right {
            Object::Int(x) if (x == &0i64 && !*b) || (x == &1i64 && *b) => true,
            Object::Float(x) if (x == &0f64 && !*b) || (x == &1f64 && *b) => true,
            Object::Bool(x) => !(b ^ x),
            _ => false,
         },
         Object::String(a) => {
            if let Object::String(s) = right {
               a == s
            } else {
               false
            }
         }
         Object::Array(a) => {
            if let Object::Array(t) = right {
               obj_vectors_equal(&a.borrow(), &t.borrow())
            } else {
               false
            }
         }
         Object::Tuple(a) => {
            if let Object::Tuple(t) = right {
               obj_vectors_equal(a, t)
            } else {
               false
            }
         }
         Object::Range(a) => {
            if let Object::Range(r) = right {
               // If the ranges match in boundaries,
               // then they are equal in value.
               a.min == r.min && a.max == r.max
            } else {
               false
            }
         }
         Object::ClassField(c1) => {
            if let Object::ClassField(c2) = right {
               c1.value.equals(&c2.value)
            } else {
               c1.value.equals(right)
            }
         }
         Object::Null => matches!(right, Object::Null),
         _ => false,
      }
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
               format!("<Method '{}' in '{}'>", name, inner.receiver.borrow().class.name)
            };

            fmt::Display::fmt(&str, f)
         }
         Object::Class(ref inner) => fmt::Display::fmt(&format!("<Class '{}'>", inner.name), f),
         Object::Instance(ref inner) => {
            fmt::Display::fmt(&format!("<Instance of '{}'>", inner.borrow().class.name), f)
         }
         Object::ClassField(ref c) => fmt::Display::fmt(c.value.as_ref(), f),
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
