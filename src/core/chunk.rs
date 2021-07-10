use crate::core::bytecode::OpCode;
use crate::objects::Object;
use num_traits::FromPrimitive;

/// Contains all the necessary information about
/// the instructions to be executed.
#[derive(Clone)]
pub struct Chunk {
   instructions: Vec<u8>,
   locations: Vec<(usize, usize)>,
   constants: Vec<Object>,
}

impl Chunk {
   /// Creates a new chunk.
   pub fn new() -> Self {
      Self {
         instructions: vec![],
         locations: vec![],
         constants: vec![],
      }
   }

   /// Adds a constant to this chunk's constants pool
   ///
   /// # Parameters
   /// - `obj`: The object to be added to the pool.
   ///
   /// # Returns
   /// - `Result<u16, ()>`: If the object was successfully added to the pool,
   /// returns the position of the object in the pool. If the item could not be
   /// added because the pool is full, returns error.
   pub fn add_constant(&mut self, obj: Object) -> Result<u16, ()> {
      return if self.constants.len() < (u16::MAX as usize) {
         // Having to create an iterator, then enumerating that iterator, to finally
         // look for the object seems very expensive. Can we do better? Do the benefits
         // of storing a single object in the heap outweigh the cost of these operations?
         // TODO: Find a better way of doing this, or do research to see if the benefits outweigh the cost.
         match self.constants.iter().enumerate().find(|x| x.1 == &obj) {
            Some(x) => Ok(x.0 as u16),
            None => {
               self.constants.push(obj);
               Ok((self.constants.len() as u16) - 1)
            }
         }
      } else {
         Err(())
      };
   }

   /// Retrieves a constant from this chunk's constants pool.
   ///
   /// # Parameters
   /// - `idx`: The index of the object constant.
   ///
   /// # Returns
   /// `&Object`: A reference to the object at the given index in the constant pool.
   pub fn get_constant(&self, idx: usize) -> &Object {
      &self.constants[idx]
   }

   /// Gets the OpCode associated with a byte instruction in the instructions list.
   pub fn get_op_code(&self, idx: usize) -> OpCode {
      FromPrimitive::from_u8(self.instructions[idx]).unwrap()
   }

   /// Adds a raw byte instruction from a given OpCode into the instructions list.
   pub fn push_op_code(&mut self, val: OpCode) {
      self.instructions.push(val as u8);
   }

   /// Gets a raw byte from the instructions list.
   pub fn get_byte(&self, idx: usize) -> u8 {
      self.instructions[idx]
   }

   /// Adds a raw byte into the instructions list.
   pub fn push_byte(&mut self, val: u8) {
      self.instructions.push(val);
   }

   /// Retrieves the current and next bytes at the given instructions list index, then converts
   /// and returns those two bytes into an u16 short.
   pub fn get_short(&self, idx: usize) -> u16 {
      let b1 = self.instructions[idx];
      let b2 = self.instructions[idx + 1];

      u16::from_be_bytes([b1, b2])
   }

   /// Splits a 16-bit integer into two bytes, and adds each individual byte in sequence into
   /// the instructions list
   pub fn push_short(&mut self, val: u16) {
      let short = val.to_be_bytes();

      self.instructions.push(short[0]);
      self.instructions.push(short[1]);
   }

   /// Modifies the byte value at the specified chunk index.
   pub fn modify_byte(&mut self, idx: usize, new_byte: u8) {
      self.instructions[idx] = new_byte;
   }

   /// Gets the line info associated with the byte at the given instruction index.
   pub fn get_line_info(&self, idx: usize) -> &(usize, usize) {
      &self.locations[idx]
   }

   /// Pushes the line info associated with the last byte in the instruction list.
   pub fn push_line_info(&mut self, line_info: (usize, usize)) {
      self.locations.push(line_info);
   }

   /// Gets the size of the instructions list.
   pub fn len(&self) -> usize {
      self.instructions.len()
   }

   /// Gets the size of the constants pool list.
   #[cfg(test)]
   pub fn get_pool_size(&self) -> usize {
      self.constants.len()
   }
}
