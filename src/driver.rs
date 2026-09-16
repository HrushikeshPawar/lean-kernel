use serde_json::Value;
use std::{io::BufRead, process::ExitCode};

use crate::{
    export,
    import::{self, reconstruct_definition},
    kernel::LeanDeclaration,
};

pub fn run(reader: impl BufRead) -> ExitCode {
    let mut state = import::ImportState::new();
    let mut checked_kind: Option<&'static str> = None;
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

                export::ExportRecord::LevelMax(record) => {
                    if let Err(error) = state.levels.add_max(&record) {
                        eprintln!("Invalid level record on line {line_number}: {error}");
                        return ExitCode::from(3);
                    }
                }

                export::ExportRecord::LevelIMax(record) => {
                    if let Err(error) = state.levels.add_imax(&record) {
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
                                    checked_kind = Some("Definition");

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
                                    checked_kind = Some("Axiom");

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

                export::ExportRecord::Theorem(record) => {
                    let reconstructed_theorem =
                        import::reconstruct_theorem(&record, &state.names, &state.exprs);

                    match reconstructed_theorem {
                        Ok(theorem) => {
                            let check_status = theorem.check_with_env(&declarations);
                            match check_status {
                                Ok(true) => {
                                    checked_kind = Some("Theorem");

                                    if declarations.iter().any(|decl| decl.name() == &theorem.name)
                                    {
                                        eprintln!("Duplicate theorem name: {:?}", theorem.name);
                                        return ExitCode::from(1);
                                    }
                                    declarations.push(LeanDeclaration::Theorem(theorem));
                                }
                                Ok(false) => {
                                    eprintln!("Theorem is invalid");
                                    return ExitCode::from(1);
                                }
                                Err(error) => {
                                    eprintln!("Unable to check theorem: {error}");
                                    return ExitCode::from(2);
                                }
                            }
                        }
                        Err(error) => {
                            eprintln!("Invalid theorem record on line {line_number}: {error}");
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

    if let Some(kind) = checked_kind {
        println!("{kind} is valid");
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

    #[test]
    fn theorem_with_non_prop_type_is_rejected() {
        // Mirrors tutorial/012_nonPropThm: type is Sort 0 (Prop), whose type is
        // Sort 1, so the theorem's type is not a proposition.
        let input = concat!(
            r#"{"in":1,"str":{"pre":0,"str":"nonPropThm"}}"#,
            "\n",
            r#"{"in":2,"str":{"pre":0,"str":"x"}}"#,
            "\n",
            r#"{"ie":0,"sort":0}"#,
            "\n",
            r#"{"bvar":0,"ie":1}"#,
            "\n",
            r#"{"forallE":{"binderInfo":"default","body":1,"name":2,"type":0},"ie":2}"#,
            "\n",
            r#"{"thm":{"all":[1],"levelParams":[],"name":1,"type":0,"value":2}}"#,
            "\n",
        );

        assert_eq!(run(Cursor::new(input)), ExitCode::from(1));
    }

    #[test]
    fn valid_theorem_stream_exits_zero() {
        // pImpliesP : ∀ (p : Prop), p → p := fun p h => h
        let input = concat!(
            // Name 1 = pImpliesP, Name 2 = p, Name 3 = h
            r#"{"in":1,"str":{"pre":0,"str":"pImpliesP"}}"#,
            "\n",
            r#"{"in":2,"str":{"pre":0,"str":"p"}}"#,
            "\n",
            r#"{"in":3,"str":{"pre":0,"str":"h"}}"#,
            "\n",
            // Expr 0 = Prop
            r#"{"ie":0,"sort":0}"#,
            "\n",
            // Expr 1 = BVar 0 (inner arrow domain: p)
            r#"{"bvar":0,"ie":1}"#,
            "\n",
            // Expr 2 = BVar 1 (inner arrow codomain: p)
            r#"{"bvar":1,"ie":2}"#,
            "\n",
            // Expr 3 = p → p
            r#"{"forallE":{"binderInfo":"default","body":2,"name":3,"type":1},"ie":3}"#,
            "\n",
            // Expr 4 = ∀ (p : Prop), p → p
            r#"{"forallE":{"binderInfo":"default","body":3,"name":2,"type":0},"ie":4}"#,
            "\n",
            // Expr 5 = BVar 0 (result h)
            r#"{"bvar":0,"ie":5}"#,
            "\n",
            // Expr 6 = fun h : p => h
            r#"{"lam":{"binderInfo":"default","body":5,"name":3,"type":1},"ie":6}"#,
            "\n",
            // Expr 7 = fun p : Prop => fun h : p => h
            r#"{"lam":{"binderInfo":"default","body":6,"name":2,"type":0},"ie":7}"#,
            "\n",
            r#"{"thm":{"all":[0],"levelParams":[],"name":1,"type":4,"value":7}}"#,
            "\n",
        );

        assert_eq!(run(Cursor::new(input)), ExitCode::from(0));
    }

    #[test]
    fn self_referential_theorem_is_rejected() {
        // Mirrors tutorial/014_selfProof: value references the theorem itself.
        let input = concat!(
            // Name 1 = selfProof, Name 2 = p
            r#"{"in":1,"str":{"pre":0,"str":"selfProof"}}"#,
            "\n",
            r#"{"in":2,"str":{"pre":0,"str":"p"}}"#,
            "\n",
            // Expr 0 = Prop
            r#"{"ie":0,"sort":0}"#,
            "\n",
            // Expr 1 = BVar 0 (p)
            r#"{"bvar":0,"ie":1}"#,
            "\n",
            // Expr 2 = ∀ p : Prop, p
            r#"{"forallE":{"binderInfo":"default","body":1,"name":2,"type":0},"ie":2}"#,
            "\n",
            // Expr 3 = const selfProof (not yet declared)
            r#"{"const":{"name":1,"us":[]},"ie":3}"#,
            "\n",
            r#"{"thm":{"all":[0],"levelParams":[],"name":1,"type":2,"value":3}}"#,
            "\n",
        );

        assert_eq!(run(Cursor::new(input)), ExitCode::from(1));
    }
}
