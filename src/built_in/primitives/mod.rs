use crate::built_in::primitives::int::IntClass;
use crate::built_in::NativeBoundMethod;
use crate::errors::RuntimeErrorType;
use crate::objects::{ClassObject, NativeMethodObj, Object};
use crate::virtual_machine::RuntimeResult;
use hashbrown::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

// Submodules
mod int;

/// Represents the list of primitive classes available through a Hinton program.
pub struct Primitives(pub HashMap<String, Rc<RefCell<ClassObject>>>);

/// The default implementation of a native primitives list.
impl Default for Primitives {
   fn default() -> Self {
      let mut primitives: HashMap<String, Rc<RefCell<ClassObject>>> = HashMap::new();

      // >>>>>> Primitive class definitions to be added after this line
      primitives.insert("Int".to_string(), Rc::new(RefCell::new(IntClass::default())));
      // <<<<<< Primitive class definitions to be added before this line

      Primitives(primitives)
   }
}

impl Primitives {
   /// Obtains a list of the names of the native functions
   pub fn get_names(&self) -> Vec<String> {
      self.0.keys().cloned().collect()
   }

   /// Obtains the ClassObject associated with a primitive class name.
   ///
   /// # Arguments
   /// * `name`: The name of the primitive class.
   ///
   /// # Returns:
   /// Result<Rc<RefCell<ClassObject>>, RuntimeResult>
   ///
   /// # Examples
   /// ```
   /// let primitives = Primitives::default();
   /// primitives.get_class_object_by_name("Float");
   /// ```
   pub fn get_class_object(&self, name: &str) -> Result<Rc<RefCell<ClassObject>>, RuntimeResult> {
      match self.0.get(name) {
         Some(f) => Ok(f.clone()),
         None => Err(RuntimeResult::Error {
            error: RuntimeErrorType::ReferenceError,
            message: format!("No primitive class named '{}'.", name),
         }),
      }
   }

   /// Obtains the property associated with a key name in primitive class.
   ///
   /// # Arguments
   /// * `name`: The name of the primitive class
   /// * `prop`: The property's name.
   ///
   /// # Returns
   /// Result<Object, RuntimeResult>
   ///
   /// # Examples
   ///
   /// ```
   /// let primitives = Primitives::default();
   /// primitives.get_prop_in_class("String", "to_lower");
   /// ```
   pub fn get_prop_in_class(&self, name: &str, prop: String) -> Result<Object, RuntimeResult> {
      match self.get_class_object(name) {
         Ok(c) => c.borrow().get(prop),
         Err(e) => Err(e),
      }
   }
}

pub trait HTPrimitive {
   /// Gets the name class name of this Hinton primitive.
   fn name(&self) -> String;

   /// Gets the non-static members of this Hinton primitive.
   fn members(&mut self) -> &mut HashMap<String, Object>;

   /// Binds a non-static method to this Hinton primitive.
   ///
   /// # Arguments
   /// * `name`: The name of the method.
   /// * `arity`: The method's arity.
   /// * `body`: The method's body.
   ///
   /// # Returns:
   /// ()
   ///
   /// # Examples
   /// ```
   /// pub type IntClass = ClassObject;
   ///
   /// impl HTPrimitive for IntClass {
   ///     // ... Implementation methods ...
   /// }
   ///
   /// IntClass.bind_method("to_string", (0, 0), int_to_string as NativeBoundMethod);
   /// ```
   fn bind_method(&mut self, name: &str, arity: (u8, u8), body: NativeBoundMethod) {
      let class_name = self.name();

      self.members().insert(
         name.to_string(),
         Object::from(NativeMethodObj {
            class_name,
            method_name: name.to_string(),
            value: Box::from(Object::Null),
            min_arity: arity.0,
            max_arity: arity.1,
            body,
         }),
      );
   }
}
