use serde_json::Value;
use std::{
    io::{self, BufRead},
    process::ExitCode,
};

mod export;

fn main() -> ExitCode {
    let stdin = io::stdin();

    for (line_index, line_result) in stdin.lock().lines().enumerate() {
        let Some(line_number) = line_index.checked_add(1) else {
            eprintln!("Overflow at line index: {line_index}");
            return ExitCode::from(3);
        };

        let line = match line_result {
            Ok(line) => line,
            Err(error) => {
                eprintln!("I/O error on line {line_number}: {error}");
                return ExitCode::from(3);
            }
        };

        if line.trim().is_empty() {
            continue;
        }

        let value: Value = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(error) => {
                eprintln!("Invalid JSON on line {line_number}: {error}");
                return ExitCode::from(3);
            }
        };

        match export::parse_supported_record(value) {
            Ok(Some(record)) => {
                eprintln!("line {line_number}: {record:?}");
            }

            Ok(None) => {
                eprintln!("Unsupported export record on line {line_number}");
                return ExitCode::from(2);
            }

            Err(error) => {
                eprintln!("Malformed export record on line {line_number}: {error}");
                return ExitCode::from(3);
            }
        }
    }

    // Input was syntactically understandable, but we are not yet claiming that we can check it.
    ExitCode::from(2)
}
