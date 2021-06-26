use std::path::PathBuf;

use crate::{compiler::Compiler, parser::Parser};

#[test]
fn base_func_has_no_arity() {
   let program = match Parser::parse("") {
      Ok(ast) => ast,
      Err(_) => panic!("Parser Had Errors."),
   };

   match Compiler::compile_ast(&PathBuf::new(), &program, vec![]) {
      Ok(res) => {
         if res.min_arity != 0u8 && res.max_arity != 0u8 {
            panic!("Base function in script should have 0 parameters.")
         }
      }
      Err(_) => panic!("Compiler Had Errors."),
   }
}

#[test]
fn base_func_has_no_defaults() {
   let program = match Parser::parse("") {
      Ok(ast) => ast,
      Err(_) => panic!("Parser Had Errors."),
   };

   match Compiler::compile_ast(&PathBuf::new(), &program, vec![]) {
      Ok(res) => {
         if !res.defaults.is_empty() {
            panic!("Base function in script should have 0 default parameters.")
         }
      }
      Err(_) => panic!("Compiler Had Errors."),
   }
}

#[test]
fn test_const_pool_no_duplicate_items() {
   let src = "8.9;".repeat(500);

   let program = match Parser::parse(src.as_str()) {
      Ok(ast) => ast,
      Err(_) => panic!("Parser Had Errors."),
   };

   match Compiler::compile_ast(&PathBuf::new(), &program, vec![]) {
      Ok(res) => {
         if res.chunk.get_pool_size() != 1 {
            panic!("Items in the constant pool should not be duplicated.")
         }
      }
      Err(_) => panic!("Compiler Had Errors."),
   }
}

#[test]
fn allow_break_inside_compact_while_loop() {
   let program = match Parser::parse("while (true) break;") {
      Ok(ast) => ast,
      Err(_) => panic!("Parser Had Errors."),
   };

   if Compiler::compile_ast(&PathBuf::new(), &program, vec![]).is_err() {
      panic!("Compiler should allow break statements inside of compact while loops.")
   }
}

#[test]
fn allow_break_inside_compact_for_loop() {
   let program = match Parser::parse("for (var x in 0..10) break;") {
      Ok(ast) => ast,
      Err(_) => panic!("Parser Had Errors."),
   };

   if Compiler::compile_ast(&PathBuf::new(), &program, vec![]).is_err() {
      panic!("Compiler should allow break statements inside of compact for loops.")
   }
}

#[test]
fn allow_break_inside_nested_while_loop_scope() {
   let src = "while true {
        {
            {{ break; }}
        }
    }";

   let program = match Parser::parse(src) {
      Ok(ast) => ast,
      Err(_) => panic!("Parser Had Errors."),
   };

   if Compiler::compile_ast(&PathBuf::new(), &program, vec![]).is_err() {
      panic!("Compiler should allow break statements inside of nested while loop scopes.")
   }
}

#[test]
fn allow_break_inside_nested_for_loop_scope() {
   let src = "for var x in 0..10 {
        {
            {{ break; }}
        }
    }";

   let program = match Parser::parse(src) {
      Ok(ast) => ast,
      Err(_) => panic!("Parser Had Errors."),
   };

   if Compiler::compile_ast(&PathBuf::new(), &program, vec![]).is_err() {
      panic!("Compiler should allow break statements inside of nested for loop scopes.")
   }
}

#[test]
fn error_if_break_outside_loop() {
   let program = match Parser::parse("func my_func() { break; }") {
      Ok(ast) => ast,
      Err(_) => panic!("Parser Had Errors."),
   };

   if Compiler::compile_ast(&PathBuf::new(), &program, vec![]).is_ok() {
      panic!("Compiler should emit error when breaking outside of a loop.")
   }
}

#[test]
fn error_if_break_inside_func_inside_loop() {
   let program = match Parser::parse("while true { func my_func() { break; } }") {
      Ok(ast) => ast,
      Err(_) => panic!("Parser Had Errors."),
   };

   if Compiler::compile_ast(&PathBuf::new(), &program, vec![]).is_ok() {
      panic!("Compiler should emit error when breaking outside inside a function inside a loop.")
   }
}

#[test]
fn error_if_return_outside_func() {
   let program = match Parser::parse("while true { return false; }") {
      Ok(ast) => ast,
      Err(_) => panic!("Parser Had Errors."),
   };

   if Compiler::compile_ast(&PathBuf::new(), &program, vec![]).is_ok() {
      panic!("Compiler should emit error when returning from outside of function.")
   }
}

#[test]
fn allow_return_inside_loop_inside_func() {
   let program = match Parser::parse("func my_func(x) { while x { return false; } }") {
      Ok(ast) => ast,
      Err(_) => panic!("Parser Had Errors."),
   };

   if Compiler::compile_ast(&PathBuf::new(), &program, vec![]).is_err() {
      panic!("Compiler should allow returning from loop inside function.")
   }
}

#[test]
fn functions_have_access_to_global_vars() {
   let src = "
        var global = \"some value\";

        func my_function() {
            print(global);
        }
    ";

   let program = match Parser::parse(src) {
      Ok(ast) => ast,
      Err(_) => panic!("Parser Had Errors."),
   };

   if Compiler::compile_ast(&PathBuf::new(), &program, vec![String::from("print")]).is_err() {
      panic!("Functions should have access to global declarations.")
   }
}
