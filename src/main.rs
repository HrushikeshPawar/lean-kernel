use serde_json::Value;
use std::{
    io::{self, BufRead},
    process::ExitCode,
};

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
                eprintln!("Invalide JSON on line {line_number}: {error}");
                return ExitCode::from(3);
            }
        };

        let Some(object) = value.as_object() else {
            eprintln!("Expected JSON object on line {line_number}");
            return ExitCode::from(3);
        };

        let keys: Vec<&str> = object.keys().map(String::as_str).collect();
        eprintln!("line {line_number}: {keys:?}");
    }

    // Input was syntatically understandable, but we are not yet claiming that we can check it.
    ExitCode::from(2)
}
