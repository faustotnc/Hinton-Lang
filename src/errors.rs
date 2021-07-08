use crate::virtual_machine::{RuntimeResult, VM};
use std::path::Path;

/// Represents an error generated by the parser or the compiler.
pub struct ErrorReport {
   /// The source line of the error.
   pub line: usize,
   /// The source column of the error.
   pub column: usize,
   /// The number of characters in the token(s) lexeme(s) that caused the error.
   pub lexeme_len: usize,
   /// The error message to display for this error report.
   pub message: String,
}

/// Represents the types of errors that can occur during
/// execution of the compiled bytecode.
pub enum RuntimeErrorType {
   ArgumentError,
   AssertionError,
   IndexError,
   InstanceError,
   Internal,
   KeyError,
   RecursionError,
   ReferenceError,
   StopIteration,
   TypeError,
   ZeroDivision,
}

/// Represents the types of errors that can occur during compilation
/// of the abstract syntax tree into bytecode.
pub enum CompilerErrorType {
   MaxCapacity,
   Reassignment,
   Reference,
   Syntax,
   Duplication,
}

/// Represents the types of errors that can occur while performing
/// some operation between Hinton objects.
pub enum ObjectOprErrType {
   TypeError(String),
   IndexError(String),
   ZeroDivisionError(String),
   KeyError(String),
}

impl ObjectOprErrType {
   /// Converts an Object Operation Error into a Runtime Result Error.
   pub fn to_runtime_error(&self) -> RuntimeResult {
      match self {
         ObjectOprErrType::TypeError(msg) => RuntimeResult::Error {
            error: RuntimeErrorType::TypeError,
            message: msg.to_owned(),
         },
         ObjectOprErrType::IndexError(msg) => RuntimeResult::Error {
            error: RuntimeErrorType::IndexError,
            message: msg.to_owned(),
         },
         ObjectOprErrType::ZeroDivisionError(msg) => RuntimeResult::Error {
            error: RuntimeErrorType::ZeroDivision,
            message: msg.to_owned(),
         },
         ObjectOprErrType::KeyError(msg) => RuntimeResult::Error {
            error: RuntimeErrorType::KeyError,
            message: msg.to_string(),
         },
      }
   }
}

/// Reports an error list coming from the parser or compiler.
///
/// # Parameters
/// - `filepath`: The file path of where the errors occurred.
/// - `errors`: An `ErrorList` containing the errors.
/// - `source`: A reference to the source contents.
pub fn report_errors_list(filepath: &Path, errors: Vec<ErrorReport>, source: &str) {
   let source_lines: Vec<&str> = source.split('\n').collect();

   for error in errors.iter() {
      eprintln!("{}", error.message);
      print_error_source(
         filepath,
         error.line,
         error.column,
         error.lexeme_len,
         &source_lines,
      );
   }

   eprintln!("\x1b[31;1mERROR:\x1b[0m Aborted execution due to previous errors.");
}

/// Prints the filepath and a snippet of the source line associated with a parser or compiler error.
///
/// # Parameters
/// - `filepath`: The file path of where the errors occurred.
/// - `line_num`: The source line number of the error.
/// - `col`: The source column number of the error.
/// - `len`: The length of the token that produced the error.
/// - `lines`: A reference to a vector with the source lines.
fn print_error_source(filepath: &Path, line_num: usize, col: usize, len: usize, lines: &[&str]) {
   let front_pad = (f64::log10(line_num as f64).floor() + 1f64) as usize;
   let line = lines.get(line_num - 1).unwrap();

   eprintln!(
      " {}---> File '{}'.",
      "-".repeat(front_pad),
      filepath.to_str().unwrap()
   );
   print_error_snippet(line_num, col, len, line);
}

/// Prints a snippet of the source line associated with an error.
///
/// # Parameters
/// - `line_num`: The source line number of the error.
/// - `col`: The source column number of the error.
/// - `len`: The length of the token that produced the error.
/// - `src`: A reference to the source error line.
pub fn print_error_snippet(line_num: usize, col: usize, len: usize, src: &str) {
   let front_pad = (f64::log10(line_num as f64).floor() + 1f64) as usize;
   // +2 for one extra space at the front and one at the back
   let whitespace_pad_size = " ".repeat(front_pad + 2);

   // Compute the column of the error with trimmed whitespaces from the source line.
   let mut removed_whitespace = 0;
   for c in src.chars() {
      if c == ' ' {
         removed_whitespace += 1;
      } else {
         break;
      }
   }

   let col = col - removed_whitespace;
   let trimmed_source = src.trim();

   if !trimmed_source.is_empty() {
      eprintln!("{}|", whitespace_pad_size);
      eprint!(" {} | ", line_num);
      eprintln!("{}", trimmed_source);
      eprint!("{}|", whitespace_pad_size);
      eprintln!(" {}\x1b[31;1m{}\x1b[0m", " ".repeat(col), "^".repeat(len));
   }

   eprintln!()
}

/// Throws a runtime error to the console.
///
/// # Parameters
/// - `vm`: A reference to the virtual machine.
/// - `error`: The generated error.
/// - `message`: The error message to be displayed.
/// - `source`: The program's source text.
pub fn report_runtime_error(vm: &VM, error: RuntimeErrorType, message: String, source: &str) {
   let source_lines: Vec<&str> = source.split('\n').collect();

   let frame = vm.current_frame();
   let f = frame.closure.function.borrow();
   let line = f.chunk.get_line_info(frame.ip - 1);

   let error_name = match error {
      RuntimeErrorType::ArgumentError => "ArgumentError",
      RuntimeErrorType::AssertionError => "AssertionError",
      RuntimeErrorType::IndexError => "IndexError",
      RuntimeErrorType::InstanceError => "InstanceError",
      RuntimeErrorType::Internal => "InternalError",
      RuntimeErrorType::KeyError => "KeyError",
      RuntimeErrorType::RecursionError => "RecursionError",
      RuntimeErrorType::ReferenceError => "ReferenceError",
      RuntimeErrorType::StopIteration => "EndOfIterationError",
      RuntimeErrorType::TypeError => "TypeError",
      RuntimeErrorType::ZeroDivision => "ZeroDivisionError",
   };

   eprintln!("\x1b[31;1m{}:\x1b[0m\x1b[1m {}\x1b[0m", error_name, message);

   let src_line = source_lines.get(line.0 - 1).unwrap();
   print_error_snippet(line.0, line.1, 1, src_line);

   // Print stack trace
   println!("Traceback (most recent call last):");
   let mut prev_err = String::new();
   let mut repeated_line_count = 0;
   let frames_list = vm.frames_stack().iter();
   let frames_list_len = frames_list.len();

   for (i, frame) in frames_list.enumerate() {
      let func = &frame.closure.function.borrow();
      let line = func.chunk.get_line_info(frame.ip);

      let new_err;
      if func.name.starts_with('<') {
         new_err = format!("{:4}at [{}:{}] in {}", "", line.0, line.1, func.name);
      } else {
         new_err = format!("{:4}at [{}:{}] in '{}()'", "", line.0, line.1, func.name);
      }

      if prev_err == new_err {
         repeated_line_count += 1;

         if repeated_line_count < 3 {
            eprintln!("{}", new_err);
         } else {
            if i == frames_list_len - 1 {
               eprintln!(
                  "{:7}\x1b[1mPrevious line repeated {} more times.\x1b[0m",
                  "",
                  repeated_line_count - 2
               );
            }

            continue;
         }
      } else {
         if repeated_line_count > 0 {
            eprintln!(
               "{:7}\x1b[1mPrevious line repeated {} more times.\x1b[0m",
               "",
               repeated_line_count - 2
            );
            repeated_line_count = 0;
         }
         eprintln!("{}", new_err);
         prev_err = new_err;
      }
   }

   eprintln!("\n\x1b[31;1mERROR:\x1b[0m Aborted execution due to previous errors.");
}
