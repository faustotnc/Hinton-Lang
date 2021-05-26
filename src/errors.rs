use crate::virtual_machine::{RuntimeResult, VirtualMachine};

/// Represents an error generated by the parser or the compiler.
pub struct ErrorReport {
    pub column: usize,
    pub lexeme_len: usize,
    pub line: usize,
    pub message: String,
}

/// Represents the types of errors that can occur during
/// execution of the compiled bytecode.
pub enum RuntimeErrorType {
    IndexError,
    StopIteration,
    Internal,
    TypeError,
    ZeroDivision,
    ArgumentError,
    RecursionError,
    ReferenceError,
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
        }
    }
}

/// Reports an error list coming from the parser or compiler.
///
/// ## Arguments
/// * `filepath` – The file path of where the errors occurred.
/// * `errors` – An `ErrorList` containing the errors.
/// * `source` – A reference to the source contents.
pub fn report_errors_list(filepath: &str, errors: Vec<ErrorReport>, source: &str) {
    let source_lines: Vec<&str> = source.split("\n").collect();

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
/// ## Arguments
/// * `filepath` – The file path of where the errors occurred.
/// * `line_num` – The source line number of the error.
/// * `col` – The source column number of the error.
/// * `len` – The length of the token that produced the error.
/// * `lines` – A reference to a vector with the source lines.
fn print_error_source(filepath: &str, line_num: usize, col: usize, len: usize, lines: &Vec<&str>) {
    let front_pad = (f64::log10(line_num as f64).floor() + 1f64) as usize;
    let line = lines.get(line_num - 1).unwrap();

    eprintln!(" {}---> File '{}'.", "-".repeat(front_pad), filepath);
    print_error_snippet(line_num, col, len, line);
}

/// Prints a snippet of the source line associated with an error.
///
/// ## Arguments
/// * `line_num` – The source line number of the error.
/// * `col` – The source column number of the error.
/// * `len` – The length of the token that produced the error.
/// * `src` – A reference to a the source error line.
pub fn print_error_snippet(line_num: usize, col: usize, len: usize, src: &str) {
    let front_pad = (f64::log10(line_num as f64).floor() + 1f64) as usize;
    // +2 for one extra space at the front and one at the back
    let whitespace_pad_size = " ".repeat(front_pad + 2);

    // Compute the line colum of the error with
    // timed whitespaces from the source line.
    let mut removed_whitespace = 0;
    for c in src.chars() {
        if c == ' ' {
            removed_whitespace += 1;
        } else {
            break;
        }
    }
    let col = col - removed_whitespace;

    eprintln!("{}|", whitespace_pad_size);
    eprint!(" {} | ", line_num);
    eprintln!("{}", src.trim());
    eprint!("{}|", whitespace_pad_size);
    eprintln!(" {}\x1b[31;1m{}\x1b[0m\n", " ".repeat(col), "^".repeat(len));
}

/// Throws a runtime error to the console
pub fn report_runtime_error(
    vm: &VirtualMachine,
    error: RuntimeErrorType,
    message: String,
    source: &str,
) {
    let source_lines: Vec<&str> = source.split("\n").collect();

    let frame = vm.current_frame();
    let line = frame.function.chunk.get_line_info(frame.ip - 1).unwrap();

    let error_name = match error {
        RuntimeErrorType::IndexError => "IndexError",
        RuntimeErrorType::StopIteration => "EndOfIterationError",
        RuntimeErrorType::Internal => "InternalError",
        RuntimeErrorType::TypeError => "TypeError",
        RuntimeErrorType::ZeroDivision => "ZeroDivisionError",
        RuntimeErrorType::ArgumentError => "ArgumentError",
        RuntimeErrorType::RecursionError => "RecursionError",
        RuntimeErrorType::ReferenceError => "ReferenceError",
    };

    eprintln!("\x1b[31;1m{}:\x1b[0m\x1b[1m {}\x1b[0m", error_name, message);

    let src_line = source_lines.get(line.0 - 1).unwrap();
    print_error_snippet(line.0, line.1, 1, src_line);

    // Print stack trace
    println!("Traceback (most recent call last):");
    let mut prev_err = String::new();
    let mut repeated_line_count = 0;
    let frames_list = vm.frames_list().iter();
    let frames_list_len = frames_list.len();

    for (i, frame) in frames_list.enumerate() {
        let func = &frame.function;
        let line = frame.function.chunk.get_line_info(frame.ip).unwrap();

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
