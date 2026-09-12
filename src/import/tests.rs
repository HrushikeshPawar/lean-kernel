use super::*;
use crate::export::{
    BinderInfo, DefinitionHints, DefinitionRecord, ExprBVarRecord, ExprForallPayload,
    ExprForallRecord, ExprSortRecord, LevelSuccRecord, NameNumPayload, NameNumRecord,
    NameStrPayload, NameStrRecord, Safety,
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
