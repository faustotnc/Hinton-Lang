#![allow(dead_code)]

#[macro_use]
extern crate num_derive;

#[cfg(feature = "bench_time")]
use std::time::Instant;

// Using other modules
use std::{env, fs, io::ErrorKind, time::Duration};

// Declaring crate-level Modules
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

// Using create-level sub-modules
use virtual_machine::InterpretResult;
use virtual_machine::VirtualMachine;

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

    // Where in the args list do we specify the filename
    let mut file_name_idx = 1;

    // Get program flags
    for arg_pos in 1..(args.len() + 1) {
        let arg = &args[arg_pos];

        if arg.to_string().starts_with("--") {
            _self.flags.push(arg.to_lowercase());
            file_name_idx += 1;
        } else {
            break;
        }
    }

    // Get the name of the file to run
    let file_name = &args[file_name_idx];

    // Get the program args
    _self.args = args[(file_name_idx + 1)..].to_vec();

    // Run the appropriate command
    match file_name.as_str() {
        "compile" => todo!("Compile command is not yet supported."),
        _ => run_file(file_name),
    }
}

/// Parses, compiles, and interprets a Hinton source file.
///
/// ## Arguments
/// * `filename` – The path to the file to run.
fn run_file(filename: &String) {
    let filepath = match fs::canonicalize(filename) {
        Ok(path) => path,
        Err(error) => {
            match error.kind() {
                ErrorKind::NotFound => eprintln!("File '{}' not found.", filename),
                ErrorKind::PermissionDenied => eprintln!("Need permission to open '{}'.", filename),
                ErrorKind::UnexpectedEof => eprintln!("Unexpected End of file '{}'.", filename),
                _ => eprintln!("Unexpected error when opening file '{}'.", filename),
            }

            std::process::exit(70)
        }
    };

    let contents = match fs::read_to_string(filepath.clone()) {
        Ok(src) => src,
        Err(error) => {
            match error.kind() {
                ErrorKind::NotFound => eprintln!("File '{}' not found.", filename),
                ErrorKind::PermissionDenied => eprintln!("Need permission to open '{}'.", filename),
                ErrorKind::UnexpectedEof => eprintln!("Unexpected End of file '{}'.", filename),
                _ => eprintln!("Unexpected error when opening file '{}'.", filename),
            }

            std::process::exit(70)
        }
    };

    // Interprets the source contents in the VM
    let result = VirtualMachine::interpret(filepath.to_str().unwrap(), &contents);

    // Exit the interpreter with the appropriate code
    match result {
        InterpretResult::ParseError => std::process::exit(65),
        InterpretResult::CompileError => std::process::exit(65),
        InterpretResult::RuntimeError => std::process::exit(70),
        InterpretResult::Ok => std::process::exit(0),
    }
}

/// Executes the provided function while also calculating it's execution time.
///
/// ## Arguments
/// `executor` – The function (with no parameters) to be executed.
///
/// ## Returns
/// `(T, Duration)` – A tuple with the result of the executed function as its
/// first parameter, and the execution time of the function as its second parameter.
pub fn exec_time<T, F: Fn() -> T>(executor: F) -> (T, Duration) {
    #[cfg(feature = "bench_time")]
    {
        let start = Instant::now();
        let exec = executor();
        let time = start.elapsed();

        return (exec, time);
    }

    #[cfg(not(feature = "bench_time"))]
    (executor(), Duration::new(0, 0))
}
