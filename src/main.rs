#[macro_use]
extern crate num_derive;

use crate::virtual_machine::{InterpretResult, VirtualMachine};
use std::io::ErrorKind;
use std::{env, fs};

// Crate-level Modules
mod ast;
mod bytecode;
mod compiler;
mod errors;
mod lexer;
mod natives;
mod objects;
mod parser;
mod virtual_machine;

#[cfg(test)]
mod tests;

/// Represents the arguments passed to the Hinton CLI.
struct CLI {
   flags: Vec<String>,
   args: Vec<String>,
}

// Static things
static FRAMES_MAX: usize = 1000;

/// The main function
fn main() {
   // structure: hinton <flags?> <filename> <program args?>
   let args: Vec<String> = env::args().collect();

   // new CLI
   let mut _self = CLI {
      flags: vec![],
      args: vec![],
   };

   // If no arguments are provided, run the REPL
   if args.len() <= 1 {
      todo!("We need a REPL!!!")
   }

   // The argument position for the filename
   let mut file_name_arg_pos = 1;

   // Get program flags
   for arg_pos in 1..(args.len() + 1) {
      let arg = &args[arg_pos];

      if arg.to_string().starts_with("--") {
         _self.flags.push(arg.to_lowercase());
         file_name_arg_pos += 1;
      } else {
         break;
      }
   }

   // Get the name of the file to run
   let file_name = &args[file_name_arg_pos];

   // Get the program args
   _self.args = args[(file_name_arg_pos + 1)..].to_vec();

   // Run the appropriate command
   match file_name.as_str() {
      "compile" => todo!("Compile command is not yet supported."),
      _ => run_file(file_name),
   }
}

/// Parses, compiles, and interprets a Hinton source file.
///
/// # Parameters
/// - `filename`: The path to the file to run.
fn run_file(filename: &str) {
   let filepath = match fs::canonicalize(filename) {
      Ok(path) => path,
      Err(error) => {
         match error.kind() {
            ErrorKind::NotFound => eprintln!("File '{}' not found.", filename),
            ErrorKind::PermissionDenied => eprintln!("Need permission to open '{}'.", filename),
            ErrorKind::UnexpectedEof => eprintln!("Unexpected end-of-file '{}'.", filename),
            _ => eprintln!("Unexpected error when opening file '{}'.", filename),
         }

         match error.raw_os_error() {
            Some(code) => std::process::exit(code),
            None => std::process::exit(70),
         }
      }
   };

   let contents = match fs::read_to_string(filepath.clone()) {
      Ok(src) => src,
      Err(error) => {
         match error.kind() {
            ErrorKind::NotFound => eprintln!("File '{}' not found.", filename),
            ErrorKind::PermissionDenied => eprintln!("Need permission to open '{}'.", filename),
            ErrorKind::UnexpectedEof => eprintln!("Unexpected end-of-file '{}'.", filename),
            _ => eprintln!("Unexpected error when opening file '{}'.", filename),
         }

         match error.raw_os_error() {
            Some(code) => std::process::exit(code),
            None => std::process::exit(70),
         }
      }
   };

   // Interprets the source contents in the VM
   let result = VirtualMachine::interpret(filepath, &contents);

   // Exit the interpreter with the appropriate code
   match result {
      InterpretResult::ParseError => std::process::exit(65),
      InterpretResult::CompileError => std::process::exit(65),
      InterpretResult::RuntimeError => std::process::exit(70),
      InterpretResult::Ok => std::process::exit(0),
   }
}
