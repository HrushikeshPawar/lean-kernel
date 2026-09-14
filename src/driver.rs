use serde_json::Value;
use std::{io::BufRead, process::ExitCode};

use crate::{
    export,
    import::{self, reconstruct_definition},
    kernel::LeanDeclaration,
};

pub fn run(reader: impl BufRead) -> ExitCode {
    let mut state = import::ImportState::new();
    let mut checked_declaration = false;
    let mut declarations = Vec::<LeanDeclaration>::new();

    for (line_index, line_result) in reader.lines().enumerate() {
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

                export::ExportRecord::LevelParam(record) => {
                    if let Err(error) = state.levels.add_param(&record, &state.names) {
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
                    if record.safety != export::Safety::Safe {
                        eprintln!("Non-safe definitions are not supported yet");
                        return ExitCode::from(2);
                    }

                    match &record.hints {
                        export::DefinitionHints::Keyword(keyword) if keyword == "opaque" => {}
                        export::DefinitionHints::Regular { .. } => {}
                        _ => {
                            eprintln!("Unsupported definition hint");
                            return ExitCode::from(2);
                        }
                    }

                    let reconstructed_definition =
                        reconstruct_definition(&record, &state.names, &state.exprs);

                    match reconstructed_definition {
                        Ok(definition) => {
                            let check_status = definition.check_with_env(&declarations);
                            match check_status {
                                Ok(true) => {
                                    checked_declaration = true;

                                    // If duplicate in definition raise
                                    if declarations
                                        .iter()
                                        .any(|decl| decl.name() == &definition.name)
                                    {
                                        eprintln!(
                                            "Duplicate definition name: {:?}",
                                            definition.name
                                        );
                                        return ExitCode::from(1);
                                    }

                                    declarations.push(LeanDeclaration::Definition(definition));
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

                export::ExportRecord::ExprLam(record) => {
                    if let Err(error) = state.exprs.add_lam(&record, &state.names) {
                        eprintln!("Invalid expression record on line {line_number}: {error}");
                        return ExitCode::from(3);
                    }
                }

                export::ExportRecord::ExprConst(record) => {
                    if let Err(error) = state.exprs.add_const(&record, &state.names, &state.levels)
                    {
                        eprintln!("Invalid expression record on line {line_number}: {error}");
                        return ExitCode::from(3);
                    }
                }

                export::ExportRecord::ExprApp(record) => {
                    if let Err(error) = state.exprs.add_app(&record) {
                        eprintln!("Invalid expression record on line {line_number}: {error}");
                        return ExitCode::from(3);
                    }
                }

                export::ExportRecord::Axiom(record) => {
                    if record.is_unsafe {
                        eprintln!("Non-safe axioms are not supported yet");
                        return ExitCode::from(2);
                    }

                    let reconstructed_axiom =
                        import::reconstruct_axiom(&record, &state.names, &state.exprs);

                    match reconstructed_axiom {
                        Ok(axiom) => {
                            let check_status = axiom.check_with_env(&declarations);
                            match check_status {
                                Ok(false) => {
                                    eprintln!("Axiom is invalid");
                                    return ExitCode::from(1);
                                }
                                Err(error) => {
                                    eprintln!("Unable to check axiom: {error}");
                                    return ExitCode::from(2);
                                }
                                Ok(true) => {
                                    checked_declaration = true;

                                    // If duplicate in axiom raise
                                    if declarations.iter().any(|decl| decl.name() == &axiom.name) {
                                        eprintln!("Duplicate axiom name: {:?}", axiom.name);
                                        return ExitCode::from(1);
                                    }
                                    declarations.push(LeanDeclaration::Axiom(axiom));
                                }
                            }
                        }
                        Err(error) => {
                            eprintln!("Invalid axiom record on line {line_number}: {error}");
                            return ExitCode::from(3);
                        }
                    }
                }
            },

            Ok(None) => {
                eprintln!("Unsupported export record on line {line_number}");
                return ExitCode::from(2);
            }

            Err(error) => {
                eprintln!("Invalid record on line {line_number}: {error}");
                return ExitCode::from(3);
            }
        }
    }

    if checked_declaration {
        println!("Definition is valid");
        ExitCode::from(0)
    } else {
        ExitCode::from(2)
    }
}

#[cfg(test)]
mod tests {
    use crate::driver::run;
    use std::{io::Cursor, process::ExitCode};
    #[test]
    fn valid_definition_does_not_hide_later_unsupported_record() {
        let input = concat!(
            r#"{"in":1,"str":{"pre":0,"str":"first"}}"#,
            "\n",
            r#"{"il":1,"succ":0}"#,
            "\n",
            r#"{"ie":0,"sort":1}"#,
            "\n",
            r#"{"ie":1,"sort":0}"#,
            "\n",
            r#"{"def":{"all":[1],"hints":"opaque","levelParams":[],"name":1,"safety":"safe","type":0,"value":1}}"#,
            "\n",
            r#"{"ie":2,"mystery":0}"#,
            "\n",
        );
        assert_eq!(run(Cursor::new(input)), ExitCode::from(2));
    }

    #[test]
    fn valid_first_definition_does_not_hide_invalid_second_definition() {
        let input = concat!(
            r#"{"in":1,"str":{"pre":0,"str":"first"}}"#,
            "\n",
            r#"{"in":2,"str":{"pre":0,"str":"second"}}"#,
            "\n",
            r#"{"il":1,"succ":0}"#,
            "\n",
            r#"{"ie":0,"sort":1}"#,
            "\n",
            r#"{"ie":1,"sort":0}"#,
            "\n",
            // first : Type := Prop     -- valid
            r#"{"def":{"all":[1],"hints":"opaque","levelParams":[],"name":1,"safety":"safe","type":0,"value":1}}"#,
            "\n",
            // second : Prop := Prop    -- invalid, because Prop : Type
            r#"{"def":{"all":[2],"hints":"opaque","levelParams":[],"name":2,"safety":"safe","type":1,"value":1}}"#,
            "\n",
        );

        assert_eq!(run(Cursor::new(input)), ExitCode::from(1));
    }

    #[test]
    fn duplicate_definition_name_is_rejected() {
        let input = concat!(
            r#"{"in":1,"str":{"pre":0,"str":"dup"}}"#,
            "\n",
            r#"{"il":1,"succ":0}"#,
            "\n",
            r#"{"ie":0,"sort":1}"#,
            "\n",
            r#"{"ie":1,"sort":0}"#,
            "\n",
            // dup : Type := Prop
            r#"{"def":{"all":[1],"hints":"opaque","levelParams":[],"name":1,"safety":"safe","type":0,"value":1}}"#,
            "\n",
            // Same declaration name again.
            r#"{"def":{"all":[1],"hints":"opaque","levelParams":[],"name":1,"safety":"safe","type":0,"value":1}}"#,
            "\n",
        );

        assert_eq!(run(Cursor::new(input)), ExitCode::from(1));
    }

    #[test]
    fn definition_rejects_undeclared_universe_parameter() {
        let input = concat!(
            // Name 1 = u
            r#"{"in":1,"str":{"pre":0,"str":"u"}}"#,
            "\n",
            // Name 2 = bad
            r#"{"in":2,"str":{"pre":0,"str":"bad"}}"#,
            "\n",
            // Level 1 = u
            r#"{"il":1,"param":1}"#,
            "\n",
            // Level 2 = u + 1
            r#"{"il":2,"succ":1}"#,
            "\n",
            // Expr 0 = Sort (u + 1)
            r#"{"ie":0,"sort":2}"#,
            "\n",
            // Expr 1 = Sort u
            r#"{"ie":1,"sort":1}"#,
            "\n",
            // bad : Sort (u + 1) := Sort u
            //
            // Typing itself works:
            // Sort u : Sort (u + 1)
            //
            // BUT u is not declared by this definition.
            r#"{"def":{"all":[2],"hints":"opaque","levelParams":[],"name":2,"safety":"safe","type":0,"value":1}}"#,
            "\n",
        );

        assert_eq!(run(Cursor::new(input)), ExitCode::from(1));
    }

    #[test]
    fn definition_accepts_declared_universe_parameter() {
        let input = concat!(
            // Name 1 = u
            r#"{"in":1,"str":{"pre":0,"str":"u"}}"#,
            "\n",
            // Name 2 = poly
            r#"{"in":2,"str":{"pre":0,"str":"poly"}}"#,
            "\n",
            // Level 1 = u
            r#"{"il":1,"param":1}"#,
            "\n",
            // Level 2 = u + 1
            r#"{"il":2,"succ":1}"#,
            "\n",
            // Expr 0 = Sort (u + 1)
            r#"{"ie":0,"sort":2}"#,
            "\n",
            // Expr 1 = Sort u
            r#"{"ie":1,"sort":1}"#,
            "\n",
            // poly.{u} : Sort (u + 1) := Sort u
            //
            // Same shape as definition_rejects_undeclared_universe_parameter,
            // but levelParams declares u, so the definition is accepted.
            r#"{"def":{"all":[2],"hints":"opaque","levelParams":[1],"name":2,"safety":"safe","type":0,"value":1}}"#,
            "\n",
        );

        assert_eq!(run(Cursor::new(input)), ExitCode::from(0));
    }

    #[test]
    fn definition_accepts_regular_hints() {
        let input = concat!(
            // Name 1 = u
            r#"{"in":1,"str":{"pre":0,"str":"u"}}"#,
            "\n",
            // Name 2 = poly
            r#"{"in":2,"str":{"pre":0,"str":"poly"}}"#,
            "\n",
            // Level 1 = u
            r#"{"il":1,"param":1}"#,
            "\n",
            // Level 2 = u + 1
            r#"{"il":2,"succ":1}"#,
            "\n",
            // Expr 0 = Sort (u + 1)
            r#"{"ie":0,"sort":2}"#,
            "\n",
            // Expr 1 = Sort u
            r#"{"ie":1,"sort":1}"#,
            "\n",
            // poly.{u} : Sort (u + 1) := Sort u
            r#"{"def":{"all":[2],"hints":{"regular":1},"levelParams":[1],"name":2,"safety":"safe","type":0,"value":1}}"#,
            "\n",
        );

        assert_eq!(run(Cursor::new(input)), ExitCode::from(0));
    }

    #[test]
    fn valid_axiom_stream_exits_zero() {
        let input = concat!(
            // Name 1 = A
            r#"{"in":1,"str":{"pre":0,"str":"A"}}"#,
            "\n",
            // Level 1 = Succ(Zero), i.e. Type
            r#"{"il":1,"succ":0}"#,
            "\n",
            // Expr 0 = Sort (Succ Zero) = Type
            r#"{"ie":0,"sort":1}"#,
            "\n",
            // axiom A : Type
            r#"{"axiom":{"isUnsafe":false,"levelParams":[],"name":1,"type":0}}"#,
            "\n",
        );

        assert_eq!(run(Cursor::new(input)), ExitCode::from(0));
    }
}
