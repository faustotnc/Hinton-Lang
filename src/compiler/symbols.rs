use super::UpValue;

/// Types of symbols available in Hinton.
#[derive(Clone)]
pub enum SymbolType {
   Variable,
   Constant,
   Function,
   Class,
   Parameter,
}

/// Represents a symbol found in a particular scope.
pub enum SL {
   /// Represents the symbol and stack position
   /// of a function's local declaration.
   Local(Symbol, usize),
   /// Represents the symbol of a global declaration,
   /// and the pool position of the symbol's name.
   Global(Symbol, usize),
   /// Represents an UpValue symbol
   UpValue(UpValue, usize),
   /// Represents a native function symbol.
   Native,
   /// Represents a symbol that was found, but there
   /// was an error with its resolution.
   Error,
}

/// Represents a symbol. Used for lexical scoping.
#[derive(Clone)]
pub struct Symbol {
   /// The symbol's name.
   pub name: String,
   /// The symbol's scope depth
   pub depth: usize,
   /// The symbol's declaration type.
   pub s_type: SymbolType,
   /// Whether the declaration for this
   /// symbol has been initialized or not.
   pub is_initialized: bool,
   /// Whether the symbol has been used or not.
   pub is_used: bool,
   /// The line and column positions of the
   /// symbol in the source code.
   pub line_info: (usize, usize),
   /// Whether the declaration for this symbol
   /// has been captured by a closure or not.
   pub is_captured: bool,
}

/// Represents the list of local symbols in a particular function.
pub struct SymbolTable {
   pub symbols: Vec<Symbol>,
}

impl SymbolTable {
   /// Creates a new symbol table.
   pub fn new(symbols: Vec<Symbol>) -> Self {
      Self { symbols }
   }

   /// Gets the number of symbols in the symbol table.
   pub fn len(&self) -> usize {
      self.symbols.len()
   }

   /// Adds a new symbol to the symbol table.
   pub fn push(&mut self, symbol: Symbol) {
      self.symbols.push(symbol)
   }

   /// Removes the last symbol from the symbol table.
   pub fn pop(&mut self) -> Option<Symbol> {
      self.symbols.pop()
   }

   /// Marks the symbol at the given position as initialized.
   pub fn mark_initialized(&mut self, pos: usize) {
      self.symbols[pos].is_initialized = true;
   }

   /// Looks for a symbol with the given name in the specified scope.
   pub fn lookup(&self, name: &String, scope: usize) -> Option<&Symbol> {
      for symbol in self.symbols.iter().filter(|s| s.depth == scope) {
         if &symbol.name == name {
            return Some(symbol);
         }
      }

      None
   }

   /// Looks for a symbol in all scopes of the symbol table.
   ///
   /// # Parameters
   /// - `name`: The symbol's name.
   /// - `used`: Whether the symbol should be marked as used as not.
   /// - `captured`: If some boolean is provided, sets the `is_captured` symbol field
   /// to the value wrapped in the option.
   ///
   /// # Returns
   /// `Option<(Symbol, usize)>`: The resolved symbol and its position in the symbol
   /// table if the symbol was found.
   pub fn resolve(&mut self, name: &String, used: bool, captured: Option<bool>) -> Option<(Symbol, usize)> {
      for (idx, symbol) in self.symbols.iter_mut().enumerate().rev() {
         if &symbol.name == name {
            if used {
               symbol.is_used = true;
            }

            if let Some(c) = captured {
               symbol.is_captured = c;
            }

            return Some((symbol.clone(), idx));
         }
      }

      None
   }

   /// Pops all the symbols that have scope depth of `min_depth` or greater.
   ///
   /// # Parameters
   /// - `min_depth`: The minimum scope depth to pop the symbols.
   /// - `pop_symbols`: Whether the symbols should actually be popped from the symbol table or not.
   ///- `show_warning`: Whether to show a warning message for unused symbols or not.
   ///
   /// # Returns
   /// - `Vev<bool>`: A vector with a boolean entry for each popped symbol, where `true` means that
   /// the symbol was captured by a closure, and `false` means it was not captured by a closure.
   pub fn pop_scope(&mut self, min_depth: usize, pop_symbols: bool, show_warning: bool) -> Vec<bool> {
      // We get the ith symbol (from the back) instead of getting the `.last()` because when
      // the `pop_symbol` parameter is false, the loop may become infinite (because we are not
      // popping the symbol off the table).
      let mut ith = self.len() - 1;
      let mut popped_symbols: Vec<bool> = vec![];

      while let Some(symbol) = self.symbols.get(ith) {
         if symbol.depth < min_depth {
            break;
         }

         if !symbol.is_used && show_warning {
            println!(
               "\x1b[33;1mWarning\x1b[0m at [{}:{}] – Variable '\x1b[1m{}\x1b[0m' is never used.",
               symbol.line_info.0, symbol.line_info.1, symbol.name
            );
         }

         popped_symbols.push(symbol.is_captured);

         if pop_symbols {
            self.symbols.pop().unwrap();
         }

         if ith == 0 {
            break;
         } else {
            ith -= 1;
         }
      }

      popped_symbols
   }
}
