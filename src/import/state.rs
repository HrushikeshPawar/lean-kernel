use crate::kernel::{LeanExpr, Level, Name};

#[derive(Debug)]
pub struct NameTable {
    pub(super) names: Vec<Name>,
}

impl Default for NameTable {
    fn default() -> Self {
        Self::new()
    }
}

impl NameTable {
    pub fn new() -> Self {
        NameTable {
            names: vec![Name::Anonymous],
        }
    }
    pub fn get(&self, index: usize) -> Option<&Name> {
        self.names.get(index)
    }
}
#[derive(Debug)]
pub struct LevelTable {
    pub(super) levels: Vec<Level>,
}

impl Default for LevelTable {
    fn default() -> Self {
        Self::new()
    }
}

impl LevelTable {
    pub fn new() -> Self {
        LevelTable {
            levels: vec![Level::Zero],
        }
    }
    pub fn get(&self, id: usize) -> Option<&Level> {
        self.levels.get(id)
    }
}
#[derive(Debug)]
pub struct ExprTable {
    pub(super) exprs: Vec<LeanExpr>,
}
impl Default for ExprTable {
    fn default() -> Self {
        Self::new()
    }
}

impl ExprTable {
    pub fn new() -> Self {
        ExprTable { exprs: Vec::new() }
    }
    pub fn get(&self, id: usize) -> Option<&LeanExpr> {
        self.exprs.get(id)
    }
}
#[derive(Debug)]
pub struct ImportState {
    pub names: NameTable,
    pub levels: LevelTable,
    pub exprs: ExprTable,
}

impl Default for ImportState {
    fn default() -> Self {
        Self::new()
    }
}

impl ImportState {
    pub fn new() -> Self {
        Self {
            names: NameTable::new(),
            levels: LevelTable::new(),
            exprs: ExprTable::new(),
        }
    }
}
