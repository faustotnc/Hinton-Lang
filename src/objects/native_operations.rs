use crate::errors::ObjectOprErrType;
use crate::objects::Object;

/// Defines negation of Hinton objects.
impl std::ops::Neg for Object {
   type Output = Result<Object, ObjectOprErrType>;

   fn neg(self) -> Self::Output {
      match self {
         Object::Int(lhs) => Ok(Object::Int(-lhs)),
         Object::Float(lhs) => Ok(Object::Float(-lhs)),
         Object::Bool(lhs) if lhs => Ok(Object::Int(-1)),
         Object::Bool(lhs) if !lhs => Ok(Object::Int(0)),
         _ => {
            return Err(ObjectOprErrType::TypeError(format!(
               "Cannot negate an object of type '{}'.",
               self.type_name()
            )))
         }
      }
   }
}

/// Defines addition of Hinton objects.
impl std::ops::Add<Object> for Object {
   type Output = Result<Object, ObjectOprErrType>;

   fn add(self, rhs: Object) -> Self::Output {
      let error_msg = Err(ObjectOprErrType::TypeError(format!(
         "Operation '+' not defined for objects of type '{}' and '{}'.",
         self.type_name(),
         rhs.type_name()
      )));

      match self {
         Object::Int(lhs) => match rhs {
            Object::Int(rhs) => Ok(Object::Int(lhs + rhs)),
            Object::Float(rhs) => Ok(Object::Float(lhs as f64 + rhs)),
            Object::Bool(rhs) => Ok(Object::Int(lhs + if rhs { 1 } else { 0 })),
            Object::String(rhs) => Ok(Object::String(format!("{}{}", lhs, rhs))),
            _ => return error_msg,
         },
         Object::Float(lhs) => match rhs {
            Object::Int(rhs) => Ok(Object::Float(lhs + rhs as f64)),
            Object::Float(rhs) => Ok(Object::Float(lhs + rhs)),
            Object::Bool(rhs) => Ok(Object::Float(lhs + if rhs { 1f64 } else { 0f64 })),
            Object::String(rhs) => Ok(Object::String(format!(
               "{}{}{}",
               lhs,
               if lhs.fract() == 0.0 { ".0" } else { "" },
               rhs
            ))),
            _ => return error_msg,
         },
         Object::String(lhs) => match rhs {
            Object::Int(rhs) => Ok(Object::String(format!("{}{}", lhs, rhs))),
            Object::Float(rhs) => Ok(Object::String(format!(
               "{}{}{}",
               lhs,
               rhs,
               if rhs.fract() == 0.0 { ".0" } else { "" }
            ))),
            Object::String(rhs) => Ok(Object::String(format!("{}{}", lhs, rhs))),
            _ => return error_msg,
         },
         Object::Bool(lhs) if lhs => match rhs {
            Object::Int(rhs) => Ok(Object::Int(rhs + 1i64)),
            Object::Float(rhs) => Ok(Object::Float(rhs + 1f64)),
            Object::Bool(rhs) => Ok(Object::Int(1 + if rhs { 1 } else { 0 })),
            _ => return error_msg,
         },
         Object::Bool(lhs) if !lhs => match rhs {
            Object::Int(rhs) => Ok(Object::Int(rhs)),
            Object::Float(rhs) => Ok(Object::Float(rhs)),
            Object::Bool(rhs) => Ok(Object::Int(if rhs { 1 } else { 0 })),
            _ => return error_msg,
         },
         _ => return error_msg,
      }
   }
}

/// Defines subtraction of Hinton objects.
impl std::ops::Sub<Object> for Object {
   type Output = Result<Object, ObjectOprErrType>;

   fn sub(self, rhs: Object) -> Self::Output {
      let error_msg = Err(ObjectOprErrType::TypeError(format!(
         "Operation '-' not defined for objects of type '{}' and '{}'.",
         self.type_name(),
         rhs.type_name()
      )));

      match self {
         Object::Int(lhs) => match rhs {
            Object::Int(rhs) => Ok(Object::Int(lhs - rhs)),
            Object::Float(rhs) => Ok(Object::Float(lhs as f64 - rhs)),
            Object::Bool(rhs) => Ok(Object::Int(lhs - if rhs { 1 } else { 0 })),
            _ => return error_msg,
         },
         Object::Float(lhs) => match rhs {
            Object::Int(rhs) => Ok(Object::Float(lhs - rhs as f64)),
            Object::Float(rhs) => Ok(Object::Float(lhs - rhs)),
            Object::Bool(rhs) => Ok(Object::Float(lhs - if rhs { 1f64 } else { 0f64 })),
            _ => return error_msg,
         },
         Object::Bool(lhs) if lhs => match rhs {
            Object::Int(rhs) => Ok(Object::Int(1i64 - rhs)),
            Object::Float(rhs) => Ok(Object::Float(1f64 - rhs)),
            Object::Bool(rhs) => Ok(Object::Int(1 - if rhs { 1 } else { 0 })),
            _ => return error_msg,
         },
         Object::Bool(lhs) if !lhs => match rhs {
            Object::Int(rhs) => Ok(Object::Int(-rhs)),
            Object::Float(rhs) => Ok(Object::Float(-rhs)),
            Object::Bool(rhs) => Ok(Object::Int(if rhs { -1 } else { 0 })),
            _ => return error_msg,
         },
         _ => return error_msg,
      }
   }
}

/// Defines multiplication of Hinton objects.
impl std::ops::Mul<Object> for Object {
   type Output = Result<Object, ObjectOprErrType>;

   fn mul(self, rhs: Object) -> Self::Output {
      let error_msg = Err(ObjectOprErrType::TypeError(format!(
         "Operation '*' not defined for objects of type '{}' and '{}'.",
         self.type_name(),
         rhs.type_name()
      )));

      match self {
         Object::Int(lhs) => match rhs {
            Object::Int(rhs) => Ok(Object::Int(lhs.saturating_mul(rhs))),
            Object::Float(rhs) => Ok(Object::Float(lhs as f64 * rhs)),
            Object::Bool(rhs) => Ok(Object::Int(if rhs { lhs } else { 0 })),
            Object::String(rhs) => Ok(Object::String(rhs.repeat(lhs as usize))),
            _ => return error_msg,
         },
         Object::Float(lhs) => match rhs {
            Object::Int(rhs) => Ok(Object::Float(lhs * rhs as f64)),
            Object::Float(rhs) => Ok(Object::Float(lhs * rhs)),
            Object::Bool(rhs) => Ok(Object::Float(if rhs { lhs } else { 0f64 })),
            _ => return error_msg,
         },
         Object::String(lhs) => match rhs {
            Object::Int(rhs) => Ok(Object::String(lhs.repeat(rhs as usize))),
            _ => return error_msg,
         },
         Object::Bool(lhs) if lhs => match rhs {
            Object::Int(rhs) => Ok(Object::Int(rhs)),
            Object::Float(rhs) => Ok(Object::Float(rhs)),
            Object::Bool(rhs) => Ok(Object::Int(if rhs { 1 } else { 0 })),
            _ => return error_msg,
         },
         Object::Bool(lhs) if !lhs => match rhs {
            Object::Int(_) => Ok(Object::Int(0)),
            Object::Float(_) => Ok(Object::Float(0f64)),
            Object::Bool(_) => Ok(Object::Int(0)),
            _ => return error_msg,
         },
         _ => return error_msg,
      }
   }
}

/// Defines division of Hinton objects.
impl std::ops::Div<Object> for Object {
   type Output = Result<Object, ObjectOprErrType>;

   fn div(self, rhs: Object) -> Self::Output {
      let error_msg = Err(ObjectOprErrType::TypeError(format!(
         "Operation '/' not defined for objects of type '{}' and '{}'.",
         self.type_name(),
         rhs.type_name()
      )));

      // Divide-by-zero errors
      if rhs.is_int() && rhs.as_int().unwrap() == 0
         || rhs.is_float() && rhs.as_float().unwrap() == 0f64
         || rhs.is_bool() && !rhs.as_bool().unwrap()
      {
         return Err(ObjectOprErrType::ZeroDivisionError(String::from(
            "Cannot divide by zero.",
         )));
      }

      match self {
         // TODO: Is converting from i64 to f64 a lossy conversion?
         Object::Int(lhs) => match rhs {
            Object::Int(rhs) => Ok(Object::Float(lhs as f64 / rhs as f64)),
            Object::Float(rhs) => Ok(Object::Float(lhs as f64 / rhs)),
            Object::Bool(_) => Ok(Object::Float(lhs as f64)),
            _ => return error_msg,
         },
         Object::Float(lhs) => match rhs {
            Object::Int(rhs) => Ok(Object::Float(lhs / rhs as f64)),
            Object::Float(rhs) => Ok(Object::Float(lhs / rhs)),
            Object::Bool(_) => Ok(Object::Float(lhs as f64)),
            _ => return error_msg,
         },
         Object::Bool(lhs) if lhs => match rhs {
            Object::Int(rhs) => Ok(Object::Float(1f64 / rhs as f64)),
            Object::Float(rhs) => Ok(Object::Float(1f64 / rhs)),
            Object::Bool(_) => Ok(Object::Float(1f64)),
            _ => return error_msg,
         },
         Object::Bool(lhs) if !lhs => match rhs {
            Object::Int(_) => Ok(Object::Float(0f64)),
            Object::Float(_) => Ok(Object::Float(0f64)),
            Object::Bool(_) => Ok(Object::Float(0f64)),
            _ => return error_msg,
         },
         _ => return error_msg,
      }
   }
}

/// Defines modulo of Hinton objects.
impl std::ops::Rem<Object> for Object {
   type Output = Result<Object, ObjectOprErrType>;

   fn rem(self, rhs: Object) -> Self::Output {
      let error_msg = Err(ObjectOprErrType::TypeError(format!(
         "Operation '%' not defined for objects of type '{}' and '{}'.",
         self.type_name(),
         rhs.type_name()
      )));

      // zero-modulo errors
      if rhs.is_int() && rhs.as_int().unwrap() == 0
         || rhs.is_float() && rhs.as_float().unwrap() == 0f64
         || rhs.is_bool() && !rhs.as_bool().unwrap()
      {
         return Err(ObjectOprErrType::ZeroDivisionError(String::from(
            "Right-hand-side of modulus cannot be zero.",
         )));
      }

      match self {
         // TODO: Is converting from f64 to i64 a lossy conversion?
         Object::Int(lhs) => match rhs {
            Object::Int(rhs) => Ok(Object::Int(lhs % rhs)),
            Object::Float(rhs) => Ok(Object::Int(lhs % rhs.floor() as i64)),
            Object::Bool(_) => Ok(Object::Int(0i64)),
            _ => return error_msg,
         },
         Object::Float(lhs) => match rhs {
            Object::Int(rhs) => Ok(Object::Float(lhs % rhs as f64)),
            Object::Float(rhs) => Ok(Object::Float(lhs % rhs)),
            Object::Bool(_) => Ok(Object::Float(lhs % 1f64)),
            _ => return error_msg,
         },
         Object::Bool(lhs) if lhs => match rhs {
            Object::Int(rhs) => Ok(Object::Int(1i64 % rhs)),
            Object::Float(rhs) => Ok(Object::Float(1f64 % rhs)),
            Object::Bool(_) => Ok(Object::Int(0i64)),
            _ => return error_msg,
         },
         Object::Bool(lhs) if !lhs => match rhs {
            Object::Int(_) => Ok(Object::Int(0i64)),
            Object::Float(_) => Ok(Object::Float(0f64)),
            Object::Bool(_) => Ok(Object::Int(0i64)),
            _ => return error_msg,
         },
         _ => return error_msg,
      }
   }
}

/// Defines the bitwise-and operation of Hinton objects.
impl std::ops::BitAnd<Object> for Object {
   type Output = Result<Object, ObjectOprErrType>;

   fn bitand(self, rhs: Object) -> Self::Output {
      let error_msg = Err(ObjectOprErrType::TypeError(format!(
         "Operation '&' not defined for objects of type '{}' and '{}'.",
         self.type_name(),
         rhs.type_name()
      )));

      match self {
         Object::Int(lhs) => match rhs {
            Object::Int(rhs) => Ok(Object::Int(lhs & rhs)),
            Object::Bool(rhs) => Ok(Object::Int(lhs & if rhs { 1 } else { 0 })),
            _ => return error_msg,
         },
         Object::Bool(lhs) if lhs => match rhs {
            Object::Int(rhs) => Ok(Object::Int(if lhs { 1 } else { 0 } & rhs)),
            Object::Bool(rhs) => Ok(Object::Int(if lhs { 1 } else { 0 } & if rhs { 1 } else { 0 })),
            _ => return error_msg,
         },
         _ => return error_msg,
      }
   }
}

/// Defines the bitwise-or operation of Hinton objects.
impl std::ops::BitOr<Object> for Object {
   type Output = Result<Object, ObjectOprErrType>;

   fn bitor(self, rhs: Object) -> Self::Output {
      let error_msg = Err(ObjectOprErrType::TypeError(format!(
         "Operation '|' not defined for objects of type '{}' and '{}'.",
         self.type_name(),
         rhs.type_name()
      )));

      match self {
         Object::Int(lhs) => match rhs {
            Object::Int(rhs) => Ok(Object::Int(lhs | rhs)),
            Object::Bool(rhs) => Ok(Object::Int(lhs | if rhs { 1 } else { 0 })),
            _ => return error_msg,
         },
         Object::Bool(lhs) if lhs => match rhs {
            Object::Int(rhs) => Ok(Object::Int(if lhs { 1 } else { 0 } | rhs)),
            Object::Bool(rhs) => Ok(Object::Int(if lhs { 1 } else { 0 } | if rhs { 1 } else { 0 })),
            _ => return error_msg,
         },
         _ => return error_msg,
      }
   }
}

/// Defines the bitwise-xor operation of Hinton objects.
impl std::ops::BitXor<Object> for Object {
   type Output = Result<Object, ObjectOprErrType>;

   fn bitxor(self, rhs: Object) -> Self::Output {
      let error_msg = Err(ObjectOprErrType::TypeError(format!(
         "Operation '^' not defined for objects of type '{}' and '{}'.",
         self.type_name(),
         rhs.type_name()
      )));

      match self {
         Object::Int(lhs) => match rhs {
            Object::Int(rhs) => Ok(Object::Int(lhs ^ rhs)),
            Object::Bool(rhs) => Ok(Object::Int(lhs ^ if rhs { 1 } else { 0 })),
            _ => return error_msg,
         },
         Object::Bool(lhs) if lhs => match rhs {
            Object::Int(rhs) => Ok(Object::Int(if lhs { 1 } else { 0 } ^ rhs)),
            Object::Bool(rhs) => Ok(Object::Int(if lhs { 1 } else { 0 } ^ if rhs { 1 } else { 0 })),
            _ => return error_msg,
         },
         _ => return error_msg,
      }
   }
}

/// Defines the bitwise-shift-left operation of Hinton objects.
impl std::ops::Shl<Object> for Object {
   type Output = Result<Object, ObjectOprErrType>;

   fn shl(self, rhs: Object) -> Self::Output {
      let error_msg = Err(ObjectOprErrType::TypeError(format!(
         "Operation '<<' not defined for objects of type '{}' and '{}'.",
         self.type_name(),
         rhs.type_name()
      )));

      match self {
         Object::Int(lhs) => match rhs {
            Object::Int(rhs) => Ok(Object::Int(lhs << rhs)),
            Object::Bool(rhs) => Ok(Object::Int(lhs << if rhs { 1 } else { 0 })),
            _ => return error_msg,
         },
         Object::Bool(lhs) if lhs => match rhs {
            Object::Int(rhs) => Ok(Object::Int(if lhs { 1 } else { 0 } << rhs)),
            Object::Bool(rhs) => Ok(Object::Int(if lhs { 1 } else { 0 } << if rhs { 1 } else { 0 })),
            _ => return error_msg,
         },
         _ => return error_msg,
      }
   }
}

/// Defines the bitwise-shift-right operation of Hinton objects.
impl std::ops::Shr<Object> for Object {
   type Output = Result<Object, ObjectOprErrType>;

   fn shr(self, rhs: Object) -> Self::Output {
      let error_msg = Err(ObjectOprErrType::TypeError(format!(
         "Operation '>>' not defined for objects of type '{}' and '{}'.",
         self.type_name(),
         rhs.type_name()
      )));

      match self {
         Object::Int(lhs) => match rhs {
            Object::Int(rhs) => Ok(Object::Int(lhs >> rhs)),
            Object::Bool(rhs) => Ok(Object::Int(lhs >> if rhs { 1 } else { 0 })),
            _ => return error_msg,
         },
         Object::Bool(lhs) if lhs => match rhs {
            Object::Int(rhs) => Ok(Object::Int(if lhs { 1 } else { 0 } >> rhs)),
            Object::Bool(rhs) => Ok(Object::Int(if lhs { 1 } else { 0 } >> if rhs { 1 } else { 0 })),
            _ => return error_msg,
         },
         _ => return error_msg,
      }
   }
}

/// Defines the bitwise-not operation of Hinton objects.
/// NOTE: Rust does not have a special bitwise-not (~) operator, instead Rust uses the '!' for
/// both logical-not and bitwise-not. Using the '!' operator on a Hinton object only applies the
/// bitwise-not operation. For the logic-not operation use the `Object.is_falsey()` method.
impl std::ops::Not for Object {
   type Output = Result<Object, ObjectOprErrType>;

   fn not(self) -> Self::Output {
      match self {
         Object::Int(opr) => Ok(Object::Int(!opr)),
         Object::Bool(opr) => Ok(Object::Int(!(opr as i64))),
         _ => {
            return Err(ObjectOprErrType::TypeError(format!(
               "Operation '~' not defined for objects of type '{}'.",
               self.type_name()
            )))
         }
      }
   }
}

impl Object {
   /// Defines exponentiation of Hinton objects.
   pub fn pow(self, rhs: Object) -> Result<Object, ObjectOprErrType> {
      let error_msg = Err(ObjectOprErrType::TypeError(format!(
         "Operation '**' not defined for objects of type '{}' and '{}'.",
         self.type_name(),
         rhs.type_name()
      )));

      match self {
         // TODO: These conversions seems error-prone and slow...
         Object::Int(lhs) => match rhs {
            Object::Int(rhs) => Ok(Object::Int((lhs as f64).powf(rhs as f64) as i64)),
            Object::Float(rhs) => Ok(Object::Float((lhs as f64).powf(rhs))),
            Object::Bool(rhs) if rhs => Ok(Object::Int(lhs)),
            Object::Bool(rhs) if !rhs => Ok(Object::Int(1)),
            _ => return error_msg,
         },
         Object::Float(lhs) => match rhs {
            Object::Int(rhs) => Ok(Object::Float(lhs.powf(rhs as f64))),
            Object::Float(rhs) => Ok(Object::Float(lhs.powf(rhs))),
            Object::Bool(rhs) if rhs => Ok(Object::Float(lhs)),
            Object::Bool(rhs) if !rhs => Ok(Object::Float(1f64)),
            _ => return error_msg,
         },
         Object::Bool(lhs) if lhs => match rhs {
            Object::Int(_) => Ok(Object::Int(1i64)),
            Object::Float(_) => Ok(Object::Float(1f64)),
            Object::Bool(_) => Ok(Object::Int(1i64)),
            _ => return error_msg,
         },
         Object::Bool(lhs) if !lhs => match rhs {
            Object::Int(rhs) => Ok(Object::Int(0f64.powf(rhs as f64) as i64)),
            Object::Float(rhs) => Ok(Object::Float(0f64.powf(rhs))),
            Object::Bool(rhs) if rhs => Ok(Object::Int(0i64)),
            Object::Bool(rhs) if !rhs => Ok(Object::Int(1i64)),
            _ => return error_msg,
         },
         _ => return error_msg,
      }
   }

   /// Defines the greater-than operation of Hinton objects.
   pub fn gt(self, rhs: Object) -> Result<Object, ObjectOprErrType> {
      let error_msg = Err(ObjectOprErrType::TypeError(format!(
         "Operation '>' not defined for objects of type '{}' and '{}'.",
         self.type_name(),
         rhs.type_name()
      )));

      match self {
         Object::Int(lhs) => match rhs {
            Object::Int(rhs) => Ok(Object::Bool(lhs > rhs)),
            Object::Float(rhs) => Ok(Object::Bool((lhs as f64) > rhs)),
            Object::Bool(rhs) if rhs => Ok(Object::Bool(lhs > if rhs { 1 } else { 0 })),
            _ => return error_msg,
         },
         Object::Float(lhs) => match rhs {
            Object::Int(rhs) => Ok(Object::Bool(lhs > rhs as f64)),
            Object::Float(rhs) => Ok(Object::Bool(lhs > rhs)),
            Object::Bool(rhs) if rhs => Ok(Object::Bool(lhs > if rhs { 1f64 } else { 0f64 })),
            _ => return error_msg,
         },
         Object::Bool(lhs) => match rhs {
            Object::Int(rhs) => Ok(Object::Bool(if lhs { 1 } else { 0 } > rhs)),
            Object::Float(rhs) => Ok(Object::Bool(if lhs { 1f64 } else { 0f64 } > rhs)),
            Object::Bool(rhs) => Ok(Object::Bool(if lhs { 1 } else { 0 } > rhs as i64)),
            _ => return error_msg,
         },
         _ => return error_msg,
      }
   }

   /// Defines the greater-than-equals operation of Hinton objects.
   pub fn gteq(self, rhs: Object) -> Result<Object, ObjectOprErrType> {
      let error_msg = Err(ObjectOprErrType::TypeError(format!(
         "Operation '>=' not defined for objects of type '{}' and '{}'.",
         self.type_name(),
         rhs.type_name()
      )));

      match self {
         Object::Int(lhs) => match rhs {
            Object::Int(rhs) => Ok(Object::Bool(lhs >= rhs)),
            Object::Float(rhs) => Ok(Object::Bool((lhs as f64) >= rhs)),
            Object::Bool(rhs) if rhs => Ok(Object::Bool(lhs >= if rhs { 1 } else { 0 })),
            _ => return error_msg,
         },
         Object::Float(lhs) => match rhs {
            Object::Int(rhs) => Ok(Object::Bool(lhs >= rhs as f64)),
            Object::Float(rhs) => Ok(Object::Bool(lhs >= rhs)),
            Object::Bool(rhs) if rhs => Ok(Object::Bool(lhs >= if rhs { 1f64 } else { 0f64 })),
            _ => return error_msg,
         },
         Object::Bool(lhs) => match rhs {
            Object::Int(rhs) => Ok(Object::Bool(if lhs { 1 } else { 0 } >= rhs)),
            Object::Float(rhs) => Ok(Object::Bool(if lhs { 1f64 } else { 0f64 } >= rhs)),
            Object::Bool(rhs) => Ok(Object::Bool(if lhs { 1 } else { 0 } >= rhs as i64)),
            _ => return error_msg,
         },
         _ => return error_msg,
      }
   }

   /// Defines the less-than operation of Hinton objects.
   pub fn lt(self, rhs: Object) -> Result<Object, ObjectOprErrType> {
      let error_msg = Err(ObjectOprErrType::TypeError(format!(
         "Operation '<' not defined for objects of type '{}' and '{}'.",
         self.type_name(),
         rhs.type_name()
      )));

      match self {
         Object::Int(lhs) => match rhs {
            Object::Int(rhs) => Ok(Object::Bool(lhs < rhs)),
            Object::Float(rhs) => Ok(Object::Bool((lhs as f64) < rhs)),
            Object::Bool(rhs) if rhs => Ok(Object::Bool(lhs < if rhs { 1 } else { 0 })),
            _ => return error_msg,
         },
         Object::Float(lhs) => match rhs {
            Object::Int(rhs) => Ok(Object::Bool(lhs < rhs as f64)),
            Object::Float(rhs) => Ok(Object::Bool(lhs < rhs)),
            Object::Bool(rhs) if rhs => Ok(Object::Bool(lhs < if rhs { 1f64 } else { 0f64 })),
            _ => return error_msg,
         },
         Object::Bool(lhs) => match rhs {
            Object::Int(rhs) => Ok(Object::Bool(if lhs { 1 } else { 0 } < rhs)),
            Object::Float(rhs) => Ok(Object::Bool(if lhs { 1f64 } else { 0f64 } < rhs)),
            Object::Bool(rhs) => Ok(Object::Bool(if lhs { 1 } else { 0 } < rhs as i64)),
            _ => return error_msg,
         },
         _ => return error_msg,
      }
   }

   /// Defines the less-than-equal operation of Hinton objects.
   pub fn lteq(self, rhs: Object) -> Result<Object, ObjectOprErrType> {
      let error_msg = Err(ObjectOprErrType::TypeError(format!(
         "Operation '<=' not defined for objects of type '{}' and '{}'.",
         self.type_name(),
         rhs.type_name()
      )));

      match self {
         Object::Int(lhs) => match rhs {
            Object::Int(rhs) => Ok(Object::Bool(lhs <= rhs)),
            Object::Float(rhs) => Ok(Object::Bool((lhs as f64) <= rhs)),
            Object::Bool(rhs) if rhs => Ok(Object::Bool(lhs <= if rhs { 1 } else { 0 })),
            _ => return error_msg,
         },
         Object::Float(lhs) => match rhs {
            Object::Int(rhs) => Ok(Object::Bool(lhs <= rhs as f64)),
            Object::Float(rhs) => Ok(Object::Bool(lhs <= rhs)),
            Object::Bool(rhs) if rhs => Ok(Object::Bool(lhs <= if rhs { 1f64 } else { 0f64 })),
            _ => return error_msg,
         },
         Object::Bool(lhs) => match rhs {
            Object::Int(rhs) => Ok(Object::Bool(if lhs { 1 } else { 0 } <= rhs)),
            Object::Float(rhs) => Ok(Object::Bool(if lhs { 1f64 } else { 0f64 } <= rhs)),
            Object::Bool(rhs) => Ok(Object::Bool(if lhs { 1 } else { 0 } <= rhs as i64)),
            _ => return error_msg,
         },
         _ => return error_msg,
      }
   }
}
