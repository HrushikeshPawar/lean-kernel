use super::*;
use crate::export::{
    AxiomRecord, BinderInfo, DefinitionHints, DefinitionRecord, ExprAppPayload, ExprAppRecord,
    ExprBVarRecord, ExprConstPayload, ExprConstRecord, ExprForallPayload, ExprForallRecord,
    ExprLamPayload, ExprLamRecord, ExprSortRecord, LevelParamRecord, LevelSuccRecord,
    NameNumPayload, NameNumRecord, NameStrPayload, NameStrRecord, Safety,
};
use crate::kernel::{LeanExpr, Level, Name};

#[test]
fn name_table_starts_with_anonymous_name() {
    let table = NameTable::new();

    assert_eq!(table.get(0), Some(&Name::Anonymous));
    assert_eq!(table.get(1), None);
}

#[test]
fn reconstructs_basic_def_name() {
    let mut table = NameTable::new();

    let record = NameStrRecord {
        id: 1,
        name: NameStrPayload {
            prefix: 0,
            value: "basicDef".to_string(),
        },
    };

    assert_eq!(table.add_str(&record), Ok(()));

    assert_eq!(
        table.get(1),
        Some(&Name::Str(
            Box::new(Name::Anonymous),
            "basicDef".to_string()
        ))
    );
}

#[test]
fn name_record_rejects_unknown_prefix() {
    let mut table = NameTable::new();

    let record = NameStrRecord {
        id: 1,
        name: NameStrPayload {
            prefix: 99,
            value: "bad".to_string(),
        },
    };

    assert!(table.add_str(&record).is_err());
}

#[test]
fn name_record_rejects_non_sequential_id() {
    let mut table = NameTable::new();

    let record = NameStrRecord {
        id: 7,
        name: NameStrPayload {
            prefix: 0,
            value: "bad".to_string(),
        },
    };

    assert!(table.add_str(&record).is_err());
}

#[test]
fn reconstructs_nested_name() {
    let mut table = NameTable::new();

    let foo = NameStrRecord {
        id: 1,
        name: NameStrPayload {
            prefix: 0,
            value: "Foo".to_string(),
        },
    };

    let bar = NameStrRecord {
        id: 2,
        name: NameStrPayload {
            prefix: 1,
            value: "Bar".to_string(),
        },
    };

    assert_eq!(table.add_str(&foo), Ok(()));
    assert_eq!(table.add_str(&bar), Ok(()));

    assert_eq!(
        table.get(2),
        Some(&Name::Str(
            Box::new(Name::Str(Box::new(Name::Anonymous), "Foo".to_string(),)),
            "Bar".to_string(),
        ))
    );
}

#[test]
fn name_table_adds_num_name() {
    let mut table = NameTable::new();

    let record = NameNumRecord {
        id: 1,
        name: NameNumPayload {
            prefix: 0,
            index: 7,
        },
    };

    table.add_num(&record).unwrap();

    assert_eq!(table.get(1), Some(&Name::Num(Box::new(Name::Anonymous), 7)));
}

#[test]
fn name_table_rejects_non_sequential_num_id() {
    let mut table = NameTable::new();

    let record = NameNumRecord {
        id: 2,
        name: NameNumPayload {
            prefix: 0,
            index: 7,
        },
    };

    let error = table.add_num(&record).unwrap_err();

    assert!(error.contains("Non-sequential name record id"));
}

#[test]
fn name_table_rejects_invalid_num_prefix() {
    let mut table = NameTable::new();

    let record = NameNumRecord {
        id: 1,
        name: NameNumPayload {
            prefix: 42,
            index: 7,
        },
    };

    let error = table.add_num(&record).unwrap_err();

    assert!(error.contains("Invalid prefix index: 42"));
}

#[test]
fn level_table_starts_with_zero() {
    let table = LevelTable::new();

    assert_eq!(table.get(0), Some(&Level::Zero));
    assert_eq!(table.get(1), None);
}

#[test]
fn reconstructs_successor_of_zero() {
    let mut table = LevelTable::new();

    let record = LevelSuccRecord {
        id: 1,
        predecessor: 0,
    };

    assert_eq!(table.add_succ(&record), Ok(()));

    assert_eq!(table.get(1), Some(&Level::Succ(Box::new(Level::Zero))));
}

#[test]
fn level_record_rejects_unknown_predecessor() {
    let mut table = LevelTable::new();

    let record = LevelSuccRecord {
        id: 1,
        predecessor: 99,
    };

    assert!(table.add_succ(&record).is_err());
}

#[test]
fn level_record_rejects_non_sequential_id() {
    let mut table = LevelTable::new();

    let record = LevelSuccRecord {
        id: 5,
        predecessor: 0,
    };

    assert!(table.add_succ(&record).is_err());
}

#[test]
fn reconstructs_level_param() {
    let mut names = NameTable::new();

    names
        .add_str(&NameStrRecord {
            id: 1,
            name: NameStrPayload {
                prefix: 0,
                value: "u".to_string(),
            },
        })
        .unwrap();

    let mut levels = LevelTable::new();

    let record = LevelParamRecord { id: 1, param: 1 };

    levels.add_param(&record, &names).unwrap();

    assert_eq!(
        levels.get(1),
        Some(&Level::Param(Name::Str(
            Box::new(Name::Anonymous),
            "u".to_string(),
        )))
    );
}

#[test]
fn level_param_rejects_unknown_name() {
    let names = NameTable::new();
    let mut levels = LevelTable::new();

    let record = LevelParamRecord { id: 1, param: 99 };

    let error = levels.add_param(&record, &names).unwrap_err();

    assert_eq!(error, "Invalid parameter name index: 99");
}

#[test]
fn expr_table_starts_empty() {
    let table = ExprTable::new();

    assert_eq!(table.get(0), None);
}

#[test]
fn reconstructs_type_sort() {
    let mut levels = LevelTable::new();

    let level_record = LevelSuccRecord {
        id: 1,
        predecessor: 0,
    };

    assert_eq!(levels.add_succ(&level_record), Ok(()));

    let mut exprs = ExprTable::new();

    let expr_record = ExprSortRecord { id: 0, level: 1 };

    assert_eq!(exprs.add_sort(&expr_record, &levels), Ok(()));

    assert_eq!(
        exprs.get(0),
        Some(&LeanExpr::Sort(Level::Succ(Box::new(Level::Zero))))
    );
}

#[test]
fn reconstructs_prop_sort() {
    let levels = LevelTable::new();
    let mut exprs = ExprTable::new();

    let expr_record = ExprSortRecord { id: 0, level: 0 };

    assert_eq!(exprs.add_sort(&expr_record, &levels), Ok(()));

    assert_eq!(exprs.get(0), Some(&LeanExpr::Sort(Level::Zero)));
}

#[test]
fn sort_record_rejects_unknown_level() {
    let levels = LevelTable::new();
    let mut exprs = ExprTable::new();

    let expr_record = ExprSortRecord { id: 0, level: 99 };

    assert!(exprs.add_sort(&expr_record, &levels).is_err());
}

#[test]
fn sort_record_rejects_non_sequential_id() {
    let levels = LevelTable::new();
    let mut exprs = ExprTable::new();

    let expr_record = ExprSortRecord { id: 7, level: 0 };

    assert!(exprs.add_sort(&expr_record, &levels).is_err());
}

#[test]
fn reconstructs_basic_def_expression_table() {
    let mut levels = LevelTable::new();

    assert_eq!(
        levels.add_succ(&LevelSuccRecord {
            id: 1,
            predecessor: 0,
        }),
        Ok(())
    );

    let mut exprs = ExprTable::new();

    assert_eq!(
        exprs.add_sort(&ExprSortRecord { id: 0, level: 1 }, &levels),
        Ok(())
    );

    assert_eq!(
        exprs.add_sort(&ExprSortRecord { id: 1, level: 0 }, &levels),
        Ok(())
    );

    assert_eq!(
        exprs.get(0),
        Some(&LeanExpr::Sort(Level::Succ(Box::new(Level::Zero))))
    );

    assert_eq!(exprs.get(1), Some(&LeanExpr::Sort(Level::Zero)));
}

#[test]
fn reconstructs_nested_successor_level() {
    let mut table = LevelTable::new();

    assert_eq!(
        table.add_succ(&LevelSuccRecord {
            id: 1,
            predecessor: 0,
        }),
        Ok(())
    );

    assert_eq!(
        table.add_succ(&LevelSuccRecord {
            id: 2,
            predecessor: 1,
        }),
        Ok(())
    );

    assert_eq!(
        table.get(2),
        Some(&Level::Succ(Box::new(Level::Succ(Box::new(Level::Zero)))))
    );
}

#[test]
fn reconstructs_basic_def_declaration() {
    // Build NameTable:
    //   0 = anonymous
    //   1 = basicDef
    let mut name_table = NameTable::new();
    let name_record = NameStrRecord {
        id: 1,
        name: NameStrPayload {
            prefix: 0,
            value: "basicDef".to_string(),
        },
    };
    assert_eq!(name_table.add_str(&name_record), Ok(()));

    // Build LevelTable:
    //   0 = Zero
    //   1 = Succ(Zero)
    let mut level_table = LevelTable::new();
    let level_record = LevelSuccRecord {
        id: 1,
        predecessor: 0,
    };
    assert_eq!(level_table.add_succ(&level_record), Ok(()));

    // Build ExprTable:
    //   0 = Sort(Succ(Zero))
    //   1 = Sort(Zero)
    let mut expr_table = ExprTable::new();
    let expr_record_0 = ExprSortRecord { id: 0, level: 1 };
    let expr_record_1 = ExprSortRecord { id: 1, level: 0 };
    assert_eq!(expr_table.add_sort(&expr_record_0, &level_table), Ok(()));
    assert_eq!(expr_table.add_sort(&expr_record_1, &level_table), Ok(()));

    // Construct the real DefinitionRecord from basicDef.
    let definition_record = DefinitionRecord {
        all: vec![1],
        name: 1,
        type_id: 0,
        value_id: 1,
        hints: DefinitionHints::Keyword("opaque".to_string()),
        level_params: vec![],
        safety: Safety::Safe,
    };

    // Call reconstruct_definition(...)
    let definition = reconstruct_definition(&definition_record, &name_table, &expr_table)
        .expect("Failed to reconstruct definition");

    // Assert:
    //   name  == basicDef
    //   ty    == Sort(Succ(Zero))
    //   value == Sort(Zero)
    assert_eq!(
        definition.name,
        Name::Str(Box::new(Name::Anonymous), "basicDef".to_string())
    );

    assert_eq!(
        definition.ty,
        LeanExpr::Sort(Level::Succ(Box::new(Level::Zero)))
    );

    assert_eq!(definition.value, LeanExpr::Sort(Level::Zero));
}

#[test]
fn reconstructs_forall_expression() {
    let mut names = NameTable::new();

    names
        .add_str(&NameStrRecord {
            id: 1,
            name: NameStrPayload {
                prefix: 0,
                value: "x".to_string(),
            },
        })
        .unwrap();

    let levels = LevelTable::new();
    let mut exprs = ExprTable::new();

    exprs
        .add_sort(&ExprSortRecord { id: 0, level: 0 }, &levels)
        .unwrap();

    let record = ExprForallRecord {
        id: 1,
        forall_expr: ExprForallPayload {
            name: 1,
            type_id: 0,
            body: 0,
            binder_info: BinderInfo::Default,
        },
    };

    exprs.add_forall(&record, &names).unwrap();

    assert_eq!(
        exprs.get(1),
        Some(&LeanExpr::Pi(
            Box::new(LeanExpr::Sort(Level::Zero)),
            Box::new(LeanExpr::Sort(Level::Zero)),
        ))
    );
}

#[test]
fn forall_expression_rejects_unknown_binder_name() {
    let names = NameTable::new();
    let levels = LevelTable::new();
    let mut exprs = ExprTable::new();

    exprs
        .add_sort(&ExprSortRecord { id: 0, level: 0 }, &levels)
        .unwrap();

    let record = ExprForallRecord {
        id: 1,
        forall_expr: ExprForallPayload {
            name: 99,
            type_id: 0,
            body: 0,
            binder_info: BinderInfo::Default,
        },
    };

    assert!(exprs.add_forall(&record, &names).is_err());
}

#[test]
fn forall_expression_rejects_unknown_type_expression() {
    let mut names = NameTable::new();

    names
        .add_str(&NameStrRecord {
            id: 1,
            name: NameStrPayload {
                prefix: 0,
                value: "x".to_string(),
            },
        })
        .unwrap();

    let mut exprs = ExprTable::new();

    let record = ExprForallRecord {
        id: 0,
        forall_expr: ExprForallPayload {
            name: 1,
            type_id: 99,
            body: 99,
            binder_info: BinderInfo::Default,
        },
    };

    assert!(exprs.add_forall(&record, &names).is_err());
}

#[test]
fn forall_expression_rejects_unknown_body_expression() {
    let mut names = NameTable::new();

    names
        .add_str(&NameStrRecord {
            id: 1,
            name: NameStrPayload {
                prefix: 0,
                value: "x".to_string(),
            },
        })
        .unwrap();

    let levels = LevelTable::new();
    let mut exprs = ExprTable::new();

    exprs
        .add_sort(&ExprSortRecord { id: 0, level: 0 }, &levels)
        .unwrap();

    let record = ExprForallRecord {
        id: 1,
        forall_expr: ExprForallPayload {
            name: 1,
            type_id: 0,
            body: 99,
            binder_info: BinderInfo::Default,
        },
    };

    let error = exprs.add_forall(&record, &names).unwrap_err();

    assert_eq!(error, "Invalid forall body expression index: 99");
}

#[test]
fn reconstructs_bvar_expression() {
    let levels = LevelTable::new();
    let mut exprs = ExprTable::new();

    exprs
        .add_sort(&ExprSortRecord { id: 0, level: 0 }, &levels)
        .unwrap();

    let record = ExprBVarRecord { id: 1, index: 0 };

    exprs.add_bvar(&record).unwrap();

    assert_eq!(exprs.get(1), Some(&LeanExpr::BVar(0)));
}

#[test]
fn bvar_rejects_non_sequential_expr_id() {
    let mut exprs = ExprTable::new();

    let record = ExprBVarRecord { id: 1, index: 0 };

    let error = exprs.add_bvar(&record).unwrap_err();

    assert!(error.contains("Non-sequential expr record id"));
}

#[test]
fn bvar_index_is_not_an_expr_table_reference() {
    let levels = LevelTable::new();
    let mut exprs = ExprTable::new();

    exprs
        .add_sort(&ExprSortRecord { id: 0, level: 0 }, &levels)
        .unwrap();

    let record = ExprBVarRecord { id: 1, index: 99 };

    exprs.add_bvar(&record).unwrap();

    assert_eq!(exprs.get(1), Some(&LeanExpr::BVar(99)));
}

#[test]
fn reconstructs_lambda_expression() {
    let mut names = NameTable::new();

    names
        .add_str(&NameStrRecord {
            id: 1,
            name: NameStrPayload {
                prefix: 0,
                value: "x".to_string(),
            },
        })
        .unwrap();

    let levels = LevelTable::new();
    let mut exprs = ExprTable::new();

    // 0 = Sort(Zero), the parameter type
    exprs
        .add_sort(&ExprSortRecord { id: 0, level: 0 }, &levels)
        .unwrap();

    // 1 = BVar(0), the body
    exprs.add_bvar(&ExprBVarRecord { id: 1, index: 0 }).unwrap();

    let record = ExprLamRecord {
        id: 2,
        lambda: ExprLamPayload {
            name: 1,
            type_id: 0,
            body: 1,
            binder_info: BinderInfo::Default,
        },
    };

    exprs.add_lam(&record, &names).unwrap();

    assert_eq!(
        exprs.get(2),
        Some(&LeanExpr::Lambda(
            Box::new(LeanExpr::Sort(Level::Zero)),
            Box::new(LeanExpr::BVar(0)),
        ))
    );
}

#[test]
fn lambda_expression_rejects_unknown_binder_name() {
    let names = NameTable::new();
    let levels = LevelTable::new();
    let mut exprs = ExprTable::new();

    exprs
        .add_sort(&ExprSortRecord { id: 0, level: 0 }, &levels)
        .unwrap();

    let record = ExprLamRecord {
        id: 1,
        lambda: ExprLamPayload {
            name: 99,
            type_id: 0,
            body: 0,
            binder_info: BinderInfo::Default,
        },
    };

    assert!(exprs.add_lam(&record, &names).is_err());
}

#[test]
fn lambda_expression_rejects_unknown_type_expression() {
    let mut names = NameTable::new();

    names
        .add_str(&NameStrRecord {
            id: 1,
            name: NameStrPayload {
                prefix: 0,
                value: "x".to_string(),
            },
        })
        .unwrap();

    let mut exprs = ExprTable::new();

    // 0 = BVar(0), the body. The type index points past it.
    exprs.add_bvar(&ExprBVarRecord { id: 0, index: 0 }).unwrap();

    let record = ExprLamRecord {
        id: 1,
        lambda: ExprLamPayload {
            name: 1,
            type_id: 99,
            body: 0,
            binder_info: BinderInfo::Default,
        },
    };

    let error = exprs.add_lam(&record, &names).unwrap_err();

    assert_eq!(error, "Invalid lambda type expression index: 99");
}

#[test]
fn lambda_expression_rejects_unknown_body_expression() {
    let mut names = NameTable::new();

    names
        .add_str(&NameStrRecord {
            id: 1,
            name: NameStrPayload {
                prefix: 0,
                value: "x".to_string(),
            },
        })
        .unwrap();

    let levels = LevelTable::new();
    let mut exprs = ExprTable::new();

    // 0 = Sort(Zero), the parameter type. The body index points past it.
    exprs
        .add_sort(&ExprSortRecord { id: 0, level: 0 }, &levels)
        .unwrap();

    let record = ExprLamRecord {
        id: 1,
        lambda: ExprLamPayload {
            name: 1,
            type_id: 0,
            body: 99,
            binder_info: BinderInfo::Default,
        },
    };

    let error = exprs.add_lam(&record, &names).unwrap_err();

    assert_eq!(error, "Invalid lambda body expression index: 99");
}

#[test]
fn lambda_expression_rejects_non_sequential_expr_id() {
    let mut names = NameTable::new();

    names
        .add_str(&NameStrRecord {
            id: 1,
            name: NameStrPayload {
                prefix: 0,
                value: "x".to_string(),
            },
        })
        .unwrap();

    let levels = LevelTable::new();
    let mut exprs = ExprTable::new();

    exprs
        .add_sort(&ExprSortRecord { id: 0, level: 0 }, &levels)
        .unwrap();

    let record = ExprLamRecord {
        id: 7,
        lambda: ExprLamPayload {
            name: 1,
            type_id: 0,
            body: 0,
            binder_info: BinderInfo::Default,
        },
    };

    let error = exprs.add_lam(&record, &names).unwrap_err();

    assert!(error.contains("Non-sequential expr record id"));
}

#[test]
fn reconstructs_const_expression() {
    // Build NameTable:
    //   0 = anonymous
    //   1 = constType
    let mut names = NameTable::new();

    names
        .add_str(&NameStrRecord {
            id: 1,
            name: NameStrPayload {
                prefix: 0,
                value: "constType".to_string(),
            },
        })
        .unwrap();

    let levels = LevelTable::new();
    let mut exprs = ExprTable::new();

    let record = ExprConstRecord {
        id: 0,
        constant: ExprConstPayload {
            name: 1,
            us: vec![],
        },
    };

    exprs.add_const(&record, &names, &levels).unwrap();

    assert_eq!(
        exprs.get(0),
        Some(&LeanExpr::Const(
            Name::Str(Box::new(Name::Anonymous), "constType".to_string()),
            vec![],
        ))
    );
}

#[test]
fn const_expression_rejects_unknown_name() {
    let names = NameTable::new();
    let levels = LevelTable::new();
    let mut exprs = ExprTable::new();

    let record = ExprConstRecord {
        id: 0,
        constant: ExprConstPayload {
            name: 99,
            us: vec![],
        },
    };

    let error = exprs.add_const(&record, &names, &levels).unwrap_err();

    assert_eq!(error, "Invalid const name index: 99");
}

#[test]
fn const_expression_rejects_unknown_level_in_us() {
    let mut names = NameTable::new();

    names
        .add_str(&NameStrRecord {
            id: 1,
            name: NameStrPayload {
                prefix: 0,
                value: "constType".to_string(),
            },
        })
        .unwrap();

    let levels = LevelTable::new();
    let mut exprs = ExprTable::new();

    let record = ExprConstRecord {
        id: 0,
        constant: ExprConstPayload {
            name: 1,
            us: vec![99],
        },
    };

    let error = exprs.add_const(&record, &names, &levels).unwrap_err();

    assert_eq!(error, "Invalid const level index: 99");
}

#[test]
fn reconstructs_non_type_axiom() {
    // Build NameTable:
    //   0 = anonymous
    //   1..8 = filler names
    //   9 = nonTypeAxiom
    //   10 = constType
    let mut name_table = NameTable::new();

    for i in 1..9 {
        name_table
            .add_str(&NameStrRecord {
                id: i,
                name: NameStrPayload {
                    prefix: 0,
                    value: format!("filler{i}"),
                },
            })
            .unwrap();
    }

    name_table
        .add_str(&NameStrRecord {
            id: 9,
            name: NameStrPayload {
                prefix: 0,
                value: "nonTypeAxiom".to_string(),
            },
        })
        .unwrap();

    name_table
        .add_str(&NameStrRecord {
            id: 10,
            name: NameStrPayload {
                prefix: 0,
                value: "constType".to_string(),
            },
        })
        .unwrap();

    // Build ExprTable:
    //   0..5 = Sort(Zero) fillers
    //   6 = Const(constType, [])
    let levels = LevelTable::new();
    let mut expr_table = ExprTable::new();

    for i in 0..6 {
        expr_table
            .add_sort(&ExprSortRecord { id: i, level: 0 }, &levels)
            .unwrap();
    }

    expr_table
        .add_const(
            &ExprConstRecord {
                id: 6,
                constant: ExprConstPayload {
                    name: 10,
                    us: vec![],
                },
            },
            &name_table,
            &levels,
        )
        .unwrap();

    // The exact final record from 011.
    let axiom_record = AxiomRecord {
        is_unsafe: false,
        level_params: vec![],
        name: 9,
        type_id: 6,
    };

    let axiom = reconstruct_axiom(&axiom_record, &name_table, &expr_table)
        .expect("Failed to reconstruct axiom");

    assert_eq!(
        axiom.name,
        Name::Str(Box::new(Name::Anonymous), "nonTypeAxiom".to_string())
    );

    assert_eq!(axiom.level_params, Vec::<Name>::new());

    assert_eq!(
        axiom.ty,
        LeanExpr::Const(
            Name::Str(Box::new(Name::Anonymous), "constType".to_string()),
            vec![],
        )
    );
}

#[test]
fn axiom_reconstruction_rejects_unknown_type() {
    let mut name_table = NameTable::new();

    for i in 1..9 {
        name_table
            .add_str(&NameStrRecord {
                id: i,
                name: NameStrPayload {
                    prefix: 0,
                    value: format!("filler{i}"),
                },
            })
            .unwrap();
    }

    name_table
        .add_str(&NameStrRecord {
            id: 9,
            name: NameStrPayload {
                prefix: 0,
                value: "nonTypeAxiom".to_string(),
            },
        })
        .unwrap();

    let levels = LevelTable::new();
    let mut expr_table = ExprTable::new();

    for i in 0..6 {
        expr_table
            .add_sort(&ExprSortRecord { id: i, level: 0 }, &levels)
            .unwrap();
    }

    let axiom_record = AxiomRecord {
        is_unsafe: false,
        level_params: vec![],
        name: 9,
        type_id: 99,
    };

    let error = reconstruct_axiom(&axiom_record, &name_table, &expr_table).unwrap_err();

    assert_eq!(error, "Invalid type expression index: 99");
}

#[test]
fn lift_constant_is_unchanged() {
    let expr = LeanExpr::Const(
        Name::Str(Box::new(Name::Anonymous), "constType".to_string()),
        vec![],
    );

    assert_eq!(expr.lift(3, 2).unwrap(), expr);
}

#[test]
fn reconstructs_application_expression() {
    let levels = LevelTable::new();
    let mut exprs = ExprTable::new();

    // 0 = Sort(Zero), the function
    exprs
        .add_sort(&ExprSortRecord { id: 0, level: 0 }, &levels)
        .unwrap();

    // 1 = BVar(0), the argument
    exprs.add_bvar(&ExprBVarRecord { id: 1, index: 0 }).unwrap();

    let record = ExprAppRecord {
        id: 2,
        application: ExprAppPayload {
            function: 0,
            arg: 1,
        },
    };

    exprs.add_app(&record).unwrap();

    assert_eq!(
        exprs.get(2),
        Some(&LeanExpr::Application(
            Box::new(LeanExpr::Sort(Level::Zero)),
            Box::new(LeanExpr::BVar(0)),
        ))
    );
}

#[test]
fn application_rejects_unknown_function() {
    let levels = LevelTable::new();
    let mut exprs = ExprTable::new();

    // 0 = Sort(Zero), a valid argument reference
    exprs
        .add_sort(&ExprSortRecord { id: 0, level: 0 }, &levels)
        .unwrap();

    let record = ExprAppRecord {
        id: 1,
        application: ExprAppPayload {
            function: 99,
            arg: 0,
        },
    };

    let error = exprs.add_app(&record).unwrap_err();

    assert_eq!(error, "Invalid application function expression index: 99");
}

#[test]
fn application_rejects_unknown_argument() {
    let levels = LevelTable::new();
    let mut exprs = ExprTable::new();

    // 0 = Sort(Zero), a valid function reference
    exprs
        .add_sort(&ExprSortRecord { id: 0, level: 0 }, &levels)
        .unwrap();

    let record = ExprAppRecord {
        id: 1,
        application: ExprAppPayload {
            function: 0,
            arg: 99,
        },
    };

    let error = exprs.add_app(&record).unwrap_err();

    assert_eq!(error, "Invalid application argument expression index: 99");
}
