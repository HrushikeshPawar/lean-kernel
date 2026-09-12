use super::*;
use serde_json::Value;

#[test]
fn parses_metadata_record() {
    let json = r#"{"meta":{"exporter":{"name":"lean4export","version":"3.1.0"},"format":{"version":"3.1.0"},"lean":{"githash":"f72c35b3f637c8c6571d353742168ab66cc22c00","version":"4.29.1"}}}"#;

    let record: MetaRecord = serde_json::from_str(json).unwrap();

    assert_eq!(record.meta.exporter.name, "lean4export");
    assert_eq!(record.meta.exporter.version, "3.1.0");
    assert_eq!(record.meta.format.version, "3.1.0");
    assert_eq!(record.meta.lean.version, "4.29.1");
}

#[test]
fn unsupported_record_is_not_a_parse_error() {
    let value: Value = serde_json::from_str(r#"{"ie":99,"mystery":0}"#).unwrap();

    let result = parse_supported_record(value);

    assert_eq!(result, Ok(None));
}

#[test]
fn supported_record_with_unknown_field_is_malformed() {
    let value: Value = serde_json::from_str(r#"{"ie":0,"sort":1,"bvar":0}"#).unwrap();

    assert!(parse_supported_record(value).is_err());
}

#[test]
fn parses_name_str_record() {
    let value: Value =
        serde_json::from_str(r#"{"in":1,"str":{"pre":0,"str":"basicDef"}}"#).unwrap();

    let record = parse_supported_record(value).unwrap();

    assert_eq!(
        record,
        Some(ExportRecord::NameStr(NameStrRecord {
            id: 1,
            name: NameStrPayload {
                prefix: 0,
                value: "basicDef".to_string(),
            },
        }))
    );
}

#[test]
fn parses_name_num_record() {
    let value: Value = serde_json::from_str(r#"{"in":6,"num":{"i":0,"pre":5}}"#).unwrap();

    let record = parse_supported_record(value).unwrap();

    assert_eq!(
        record,
        Some(ExportRecord::NameNum(NameNumRecord {
            id: 6,
            name: NameNumPayload {
                prefix: 5,
                index: 0,
            },
        }))
    );
}

#[test]
fn parses_level_succ_record() {
    let value: Value = serde_json::from_str(r#"{"il":1,"succ":0}"#).unwrap();

    let record = parse_supported_record(value).unwrap();

    assert_eq!(
        record,
        Some(ExportRecord::LevelSucc(LevelSuccRecord {
            id: 1,
            predecessor: 0,
        }))
    );
}

#[test]
fn parses_expr_sort_record() {
    let value: Value = serde_json::from_str(r#"{"ie":0,"sort":1}"#).unwrap();

    let record = parse_supported_record(value).unwrap();

    assert_eq!(
        record,
        Some(ExportRecord::ExprSort(ExprSortRecord { id: 0, level: 1 }))
    );
}

#[test]
fn parses_second_expr_sort_record() {
    let value: Value = serde_json::from_str(r#"{"ie":1,"sort":0}"#).unwrap();

    let record = parse_supported_record(value).unwrap();

    assert_eq!(
        record,
        Some(ExportRecord::ExprSort(ExprSortRecord { id: 1, level: 0 }))
    );
}

#[test]
fn parses_definition_record() {
    let value: Value = serde_json::from_str(
            r#"{"def":{"all":[1],"hints":"opaque","levelParams":[],"name":1,"safety":"safe","type":0,"value":1}}"#,
        )
        .unwrap();

    let record = parse_supported_record(value).unwrap();

    assert_eq!(
        record,
        Some(ExportRecord::Definition(DefinitionRecord {
            all: vec![1],
            hints: DefinitionHints::Keyword("opaque".to_string()),
            level_params: vec![],
            name: 1,
            safety: Safety::Safe,
            type_id: 0,
            value_id: 1,
        }))
    );
}

#[test]
fn parses_regular_definition_hint() {
    let value: Value = serde_json::from_str(
            r#"{"def":{"all":[1],"hints":{"regular":2},"levelParams":[],"name":1,"safety":"safe","type":0,"value":1}}"#,
        )
        .unwrap();

    let record = parse_supported_record(value).unwrap();

    assert_eq!(
        record,
        Some(ExportRecord::Definition(DefinitionRecord {
            all: vec![1],
            hints: DefinitionHints::Regular { regular: 2 },
            level_params: vec![],
            name: 1,
            safety: Safety::Safe,
            type_id: 0,
            value_id: 1,
        }))
    );
}

#[test]
fn parses_expr_forall_record() {
    let value: Value = serde_json::from_str(
        r#"{"forallE":{"binderInfo":"default","body":1,"name":6,"type":1},"ie":2}"#,
    )
    .unwrap();

    let record = parse_supported_record(value).unwrap();

    assert_eq!(
        record,
        Some(ExportRecord::ExprForall(ExprForallRecord {
            id: 2,
            forall_expr: ExprForallPayload {
                name: 6,
                type_id: 1,
                body: 1,
                binder_info: BinderInfo::Default,
            },
        }))
    );
}

#[test]
fn parses_expr_bvar_record() {
    let value: Value = serde_json::from_str(r#"{"bvar":0,"ie":1}"#).unwrap();

    let record = parse_supported_record(value).unwrap();

    assert_eq!(
        record,
        Some(ExportRecord::ExprBVar(ExprBVarRecord { id: 1, index: 0 }))
    );
}

#[test]
fn parses_expr_lam_record() {
    let value: Value = serde_json::from_str(
        r#"{"ie":4,"lam":{"binderInfo":"default","body":3,"name":8,"type":0}}"#,
    )
    .unwrap();

    let record = parse_supported_record(value).unwrap();

    assert_eq!(
        record,
        Some(ExportRecord::ExprLam(ExprLamRecord {
            id: 4,
            lambda: ExprLamPayload {
                name: 8,
                type_id: 0,
                body: 3,
                binder_info: BinderInfo::Default,
            },
        }))
    );
}
