use serde_json::Value;
use std::{
    io::{self, BufRead},
    process::ExitCode,
};

use lean_kernel::{
    export,
    lean::{self, reconstruct_definition},
};

fn main() -> ExitCode {
    let stdin = io::stdin();
    let mut state = lean::ImportState::new();

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
            Ok(Some(record)) => match record {
                export::ExportRecord::Meta(_) => {}

                export::ExportRecord::NameStr(record) => {
                    if let Err(error) = state.names.add_str(&record) {
                        eprintln!("Invalid name record on line {line_number}: {error}");
                        return ExitCode::from(3);
                    }
                }

                export::ExportRecord::NameNum(record) => {
                    if let Err(error) = state.names.add_num(&record) {
                        eprintln!("Invalid name record on line {line_number}: {error}");
                        return ExitCode::from(3);
                    }
                }

                export::ExportRecord::LevelSucc(record) => {
                    if let Err(error) = state.levels.add_succ(&record) {
                        eprintln!("Invalid level record on line {line_number}: {error}");
                        return ExitCode::from(3);
                    }
                }

                export::ExportRecord::ExprSort(record) => {
                    if let Err(error) = state.exprs.add_sort(&record, &state.levels) {
                        eprintln!("Invalid expression record on line {line_number}: {error}");
                        return ExitCode::from(3);
                    }
                }

                export::ExportRecord::ExprForall(record) => {
                    if let Err(error) = state.exprs.add_forall(&record, &state.names) {
                        eprintln!("Invalid expression record on line {line_number}: {error}");
                        return ExitCode::from(3);
                    }
                }

                export::ExportRecord::Definition(record) => {
                    if !record.level_params.is_empty() {
                        eprintln!("Universe-polymorphic definitions are not supported yet");
                        return ExitCode::from(2);
                    }

                    if record.safety != export::Safety::Safe {
                        eprintln!("Non-safe definitions are not supported yet");
                        return ExitCode::from(2);
                    }

                    if record.hints != export::DefinitionHints::Keyword("opaque".to_string()) {
                        eprintln!("Unsupported definition hint");
                        return ExitCode::from(2);
                    }

                    let reconstructed_definition =
                        reconstruct_definition(&record, &state.names, &state.exprs);

                    match reconstructed_definition {
                        Ok(definition) => {
                            let check_status = definition.check();
                            match check_status {
                                Ok(true) => {
                                    println!("Definition is valid");
                                    return ExitCode::from(0);
                                }
                                Ok(false) => {
                                    eprintln!("Definition is invalid");
                                    return ExitCode::from(1);
                                }
                                Err(error) => {
                                    eprintln!("Unable to check definition: {error}");
                                    return ExitCode::from(2);
                                }
                            }
                        }
                        Err(error) => {
                            eprintln!("Invalid definition record on line {line_number}: {error}");
                            return ExitCode::from(3);
                        }
                    }
                }

                export::ExportRecord::ExprBVar(record) => {
                    if let Err(error) = state.exprs.add_bvar(&record) {
                        eprintln!("Invalid expression record on line {line_number}: {error}");
                        return ExitCode::from(3);
                    }
                }
            },

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
