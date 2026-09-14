use serde::Deserialize;

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
pub struct LevelParamRecord {
    #[serde(rename = "il")]
    pub id: usize,

    pub param: usize,
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
#[serde(deny_unknown_fields)]
pub struct ExprBVarRecord {
    #[serde(rename = "ie")]
    pub id: usize,

    #[serde(rename = "bvar")]
    pub index: usize,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ExprLamRecord {
    #[serde(rename = "ie")]
    pub id: usize,

    #[serde(rename = "lam")]
    pub lambda: ExprLamPayload,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ExprLamPayload {
    pub name: usize,

    #[serde(rename = "type")]
    pub type_id: usize,

    pub body: usize,

    #[serde(rename = "binderInfo")]
    pub binder_info: BinderInfo,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ExprConstRecord {
    #[serde(rename = "ie")]
    pub id: usize,

    #[serde(rename = "const")]
    pub constant: ExprConstPayload,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ExprConstPayload {
    pub name: usize,
    pub us: Vec<usize>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ExprAppRecord {
    #[serde(rename = "ie")]
    pub id: usize,

    #[serde(rename = "app")]
    pub application: ExprAppPayload,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ExprAppPayload {
    #[serde(rename = "fn")]
    pub function: usize,

    pub arg: usize,
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

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AxiomEnvelope {
    pub axiom: AxiomRecord,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AxiomRecord {
    #[serde(rename = "isUnsafe")]
    pub is_unsafe: bool,

    #[serde(rename = "levelParams")]
    pub level_params: Vec<usize>,

    pub name: usize,

    #[serde(rename = "type")]
    pub type_id: usize,
}

#[derive(Debug, PartialEq)]
pub enum ExportRecord {
    Meta(MetaRecord),
    NameStr(NameStrRecord),
    NameNum(NameNumRecord),
    LevelSucc(LevelSuccRecord),
    LevelParam(LevelParamRecord),
    ExprSort(ExprSortRecord),
    ExprForall(ExprForallRecord),
    ExprLam(ExprLamRecord),
    ExprBVar(ExprBVarRecord),
    ExprConst(ExprConstRecord),
    ExprApp(ExprAppRecord),
    Definition(DefinitionRecord),
    Axiom(AxiomRecord),
}
