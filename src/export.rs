use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MetaRecord {
    pub meta: Meta,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Meta {
    pub exporter: ExporterInfo,
    pub format: FormatInfo,
    pub lean: LeanInfo,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ExporterInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FormatInfo {
    pub version: String,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LeanInfo {
    pub githash: String,
    pub version: String,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NameStrRecord {
    #[serde(rename = "in")]
    pub id: usize,

    #[serde(rename = "str")]
    pub name: NameStrPayload,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NameStrPayload {
    #[serde(rename = "pre")]
    pub prefix: usize,

    #[serde(rename = "str")]
    pub value: String,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NameNumRecord {
    #[serde(rename = "in")]
    pub id: usize,

    #[serde(rename = "num")]
    pub name: NameNumPayload,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NameNumPayload {
    #[serde(rename = "pre")]
    pub prefix: usize,

    #[serde(rename = "i")]
    pub index: usize,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LevelSuccRecord {
    #[serde(rename = "il")]
    pub id: usize,

    #[serde(rename = "succ")]
    pub predecessor: usize,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ExprSortRecord {
    #[serde(rename = "ie")]
    pub id: usize,

    #[serde(rename = "sort")]
    pub level: usize,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ExprForallRecord {
    #[serde(rename = "ie")]
    pub id: usize,

    #[serde(rename = "forallE")]
    pub forall_expr: ExprForallPayload,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ExprForallPayload {
    pub name: usize,

    #[serde(rename = "type")]
    pub type_id: usize,

    pub body: usize,

    #[serde(rename = "binderInfo")]
    pub binder_info: BinderInfo,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum BinderInfo {
    Default,
    Implicit,
    StrictImplicit,
    InstImplicit,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DefinitionEnvelope {
    #[serde(rename = "def")]
    pub definition: DefinitionRecord,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DefinitionRecord {
    pub all: Vec<usize>,
    pub hints: DefinitionHints,

    #[serde(rename = "levelParams")]
    pub level_params: Vec<usize>,

    pub name: usize,
    pub safety: Safety,

    #[serde(rename = "type")]
    pub type_id: usize,

    #[serde(rename = "value")]
    pub value_id: usize,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Safety {
    Unsafe,
    Safe,
    Partial,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum DefinitionHints {
    Keyword(String),
    Regular { regular: usize },
}

#[derive(Debug, PartialEq)]
pub enum ExportRecord {
    Meta(MetaRecord),
    NameStr(NameStrRecord),
    NameNum(NameNumRecord),
    LevelSucc(LevelSuccRecord),
    ExprSort(ExprSortRecord),
    ExprForall(ExprForallRecord),
    Definition(DefinitionRecord),
}

pub fn parse_supported_record(value: Value) -> Result<Option<ExportRecord>, String> {
    let Some(object) = value.as_object() else {
        return Err("expected JSON object".to_string());
    };

    // println!("\n{object:?}");

    if object.contains_key("meta") {
        let record =
            serde_json::from_value::<MetaRecord>(value).map_err(|error| error.to_string())?;
        return Ok(Some(ExportRecord::Meta(record)));
    }

    if object.contains_key("str") {
        let record =
            serde_json::from_value::<NameStrRecord>(value).map_err(|error| error.to_string())?;
        return Ok(Some(ExportRecord::NameStr(record)));
    }

    if object.contains_key("num") {
        let record =
            serde_json::from_value::<NameNumRecord>(value).map_err(|error| error.to_string())?;
        return Ok(Some(ExportRecord::NameNum(record)));
    }

    if object.contains_key("succ") {
        let record =
            serde_json::from_value::<LevelSuccRecord>(value).map_err(|error| error.to_string())?;
        return Ok(Some(ExportRecord::LevelSucc(record)));
    }

    if object.contains_key("sort") {
        let record =
            serde_json::from_value::<ExprSortRecord>(value).map_err(|error| error.to_string())?;
        return Ok(Some(ExportRecord::ExprSort(record)));
    }

    if object.contains_key("forallE") {
        let record =
            serde_json::from_value::<ExprForallRecord>(value).map_err(|error| error.to_string())?;
        return Ok(Some(ExportRecord::ExprForall(record)));
    }

    if object.contains_key("def") {
        let record = serde_json::from_value::<DefinitionEnvelope>(value)
            .map_err(|error| error.to_string())?;
        return Ok(Some(ExportRecord::Definition(record.definition)));
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let value: Value = serde_json::from_str(r#"{"ie":99,"bvar":0}"#).unwrap();

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
}
