#[derive(Debug, Clone, PartialEq)]
pub enum Name {
    Anonymous,
    Str(Box<Name>, String),
    Num(Box<Name>, usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Level {
    Zero,
    Succ(Box<Level>),
    Param(Name),

    Max(Box<Level>, Box<Level>),
    IMax(Box<Level>, Box<Level>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum LeanExpr {
    Sort(Level),
    BVar(usize),
    Const(Name, Vec<Level>),
    Pi(Box<LeanExpr>, Box<LeanExpr>),
    Lambda(Box<LeanExpr>, Box<LeanExpr>),
    Application(Box<LeanExpr>, Box<LeanExpr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LeanDefinition {
    pub name: Name,
    pub level_params: Vec<Name>,
    pub ty: LeanExpr,
    pub value: LeanExpr,
}
