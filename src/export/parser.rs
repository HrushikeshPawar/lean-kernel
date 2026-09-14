use super::records::*;
use serde_json::Value;

pub fn parse_supported_record(value: Value) -> Result<Option<ExportRecord>, String> {
    let Some(object) = value.as_object() else {
        return Err("expected JSON object".to_string());
    };

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

    if object.contains_key("bvar") {
        let record =
            serde_json::from_value::<ExprBVarRecord>(value).map_err(|error| error.to_string())?;
        return Ok(Some(ExportRecord::ExprBVar(record)));
    }

    if object.contains_key("lam") {
        let record =
            serde_json::from_value::<ExprLamRecord>(value).map_err(|error| error.to_string())?;
        return Ok(Some(ExportRecord::ExprLam(record)));
    }

    if object.contains_key("const") {
        let record =
            serde_json::from_value::<ExprConstRecord>(value).map_err(|error| error.to_string())?;

        return Ok(Some(ExportRecord::ExprConst(record)));
    }

    if object.contains_key("app") {
        let record =
            serde_json::from_value::<ExprAppRecord>(value).map_err(|error| error.to_string())?;

        return Ok(Some(ExportRecord::ExprApp(record)));
    }

    if object.contains_key("succ") {
        let record =
            serde_json::from_value::<LevelSuccRecord>(value).map_err(|error| error.to_string())?;
        return Ok(Some(ExportRecord::LevelSucc(record)));
    }

    if object.contains_key("param") {
        let record =
            serde_json::from_value::<LevelParamRecord>(value).map_err(|error| error.to_string())?;

        return Ok(Some(ExportRecord::LevelParam(record)));
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

    if object.contains_key("axiom") {
        let record =
            serde_json::from_value::<AxiomEnvelope>(value).map_err(|error| error.to_string())?;
        return Ok(Some(ExportRecord::Axiom(record.axiom)));
    }

    Ok(None)
}
